/// # Example
/// - run the experiment
/// ```nushell
/// const NB_BYTES = 1_024 * 10
/// const K = 10
/// const N = 2 * $K
/// const NB_MEASUREMENTS = 1_000
/// const MEASUREMENT_SCHEDULE = 1
/// const MAX_T = 150
///
/// cargo run --example inbreeding -- ...[
///     $NB_BYTES,
///     -k $K
///     -n $N
///     --nb-measurements $NB_MEASUREMENTS
///     --measurement-schedule $MEASUREMENT_SCHEDULE
///     -t $MAX_T
///     --test-case end-to-end
/// ] | lines | into float | save --force baseline.nuon
///
/// seq 1 $K | reverse | each {|r|
///     let inbreeding = cargo run --example inbreeding -- ...[
///         $NB_BYTES,
///         -k $K
///         -n $N
///         --nb-measurements $NB_MEASUREMENTS
///         --measurement-schedule $MEASUREMENT_SCHEDULE
///         -t $MAX_T
///         --test-case recoding
///         -r $r
///     ] | lines | into float
///
///     {
///         r: $r,
///         inbreeding: $inbreeding,
///     }
/// } | save --force inbreeding.nuon
/// ```
/// - plot the results
/// ```nushell
/// let data = open inbreeding.nuon
/// let k = $data.r | math max
/// let w = 3
/// let l = $data.inbreeding.0 | length
///
/// use std repeat
///
/// # let raw = $data | update inbreeding { take ($l - $w + 1)}
/// # let smooth = $data | update inbreeding { prepend (1 | repeat $w) | window $w | each { math avg } }
/// let smooth = $data
///
/// $smooth
///     | insert name {|it|
///        let r = if $it.r == $k { "k" }  else { $"k - ($k - $it.r)" }
///        $"$\\sigma = ($r)$"
///     }
///     # | append ($raw | insert name null | insert style { line: { alpha: 0.1 } })
///     | update inbreeding {|it|
///         let l = $it.inbreeding | length
///         $it.inbreeding | wrap y | merge (seq 0 $l | wrap x) | insert e 0
///     }
///     | rename --column { inbreeding: "points" }
///     | insert style.color {|it|
///         match $it.r {
///             10 => "tab:red",
///             9 => "tab:orange",
///             8 => "tab:olive",
///             7 => "tab:blue",
///             6 => "tab:purple",
///             5 => "tab:green",
///             4 => "tab:cyan",
///             3 => "tab:brown",
///             2 => "tab:pink",
///             _ => "tab:gray",
///         }
///     }
///     | reject r
///     | save --force /tmp/graphs.json
/// ```
/// ```
/// let x_min = open /tmp/graphs.json | get points.0.x | math min
/// let x_max = open /tmp/graphs.json | get points.0.x | math max
///
/// gplt plot ...[
///     --json-data-file /tmp/graphs.json
///     --x-lim ($x_min - 1) ($x_max + 1)
///     --y-lim -0.01 1.01
///     --fullscreen
///     # --title "diversity over time when recoding shards $r$ shards"
///     --x-label "time (in nb of steps)"
///     --y-label "diversity $\\delta$"
///     --dpi 150
///     --fig-size ...[16, 5]
///     --font ({ size: 15, family: serif, sans-serif: Helvetica } | to json)
///     --use-tex
///     --legend-loc "upper right"
///     # --save inbreeding.pdf
/// ]
/// ```
use std::process::exit;

use ark_ff::PrimeField;

use clap::{Parser, ValueEnum};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use komodo::{
    error::KomodoError,
    fec::{self, Shard},
    linalg::Matrix,
};
use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng, RngCore};

fn random_bytes(n: usize, rng: &mut ThreadRng) -> Vec<u8> {
    (0..n).map(|_| rng.gen::<u8>()).collect()
}

fn setup<F: PrimeField>(bytes: &[u8], k: usize, n: usize) -> Result<Vec<Shard<F>>, KomodoError> {
    let points: Vec<F> = (0..n)
        .map(|i| F::from_le_bytes_mod_order(&i.to_le_bytes()))
        .collect();
    let encoding_mat = Matrix::vandermonde_unchecked(&points, k);
    let shards = fec::encode(bytes, &encoding_mat)?;

    Ok(shards)
}

fn measure_inbreeding<F: PrimeField>(
    shards: &[Shard<F>],
    k: usize,
    nb_measurements: usize,
    mp: &MultiProgress,
    sty: &ProgressStyle,
    rng: &mut impl RngCore,
) -> f64 {
    let mut s: Vec<_> = shards.to_vec();
    let mut count = 0;

    let pb = mp.add(ProgressBar::new(nb_measurements as u64));
    pb.set_style(sty.clone());
    pb.set_message("measure");
    for _ in 0..nb_measurements {
        // get any k of the shards
        s.shuffle(rng);
        if fec::decode(s.iter().take(k).cloned().collect()).is_ok() {
            count += 1;
        }
        pb.inc(1);
    }

    count as f64 / nb_measurements as f64
}

fn end_to_end<F, Fun>(
    bytes: &[u8],
    k: usize,
    n: usize,
    max_t: usize,
    nb_measurements: usize,
    measurement_schedule: Fun,
    rng: &mut impl RngCore,
) -> Result<(), KomodoError>
where
    F: PrimeField,
    Fun: Fn(usize) -> bool,
{
    let original_shards = setup(bytes, k, n)?;
    let mut shards = original_shards.clone();

    let mp = MultiProgress::new();
    let sty = ProgressStyle::with_template("{msg}: {bar:40.cyan/blue} {pos:>7}/{len:7}")
        .unwrap()
        .progress_chars("##-");
    let pb = mp.add(ProgressBar::new(max_t as u64));
    pb.set_style(sty.clone());
    pb.set_message("main");
    for t in 0..=max_t {
        if measurement_schedule(t) {
            let inbreeding = measure_inbreeding(&shards, k, nb_measurements, &mp, &sty, rng);
            println!("{}", inbreeding);
        }

        // decode the data
        let data = fec::decode(original_shards.clone())?;

        // re-encode a new random shard
        let encoding_mat = Matrix::vandermonde_unchecked(&[F::rand(rng)], k);
        let new_shard = fec::encode(&data, &encoding_mat)?.first().unwrap().clone();
        shards.push(new_shard);

        pb.inc(1);
    }
    pb.finish_with_message("done");

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn recoding<F, Fun>(
    bytes: &[u8],
    k: usize,
    n: usize,
    max_t: usize,
    nb_shards_to_recode: usize,
    nb_measurements: usize,
    measurement_schedule: Fun,
    rng: &mut impl RngCore,
) -> Result<(), KomodoError>
where
    F: PrimeField,
    Fun: Fn(usize) -> bool,
{
    let mut shards = setup::<F>(bytes, k, n)?;

    let mp = MultiProgress::new();
    let sty = ProgressStyle::with_template("{msg}: {bar:40.cyan/blue} {pos:>7}/{len:7}")
        .unwrap()
        .progress_chars("##-");
    let pb = mp.add(ProgressBar::new(max_t as u64));
    pb.set_style(sty.clone());
    pb.set_message("main");
    for t in 0..=max_t {
        if measurement_schedule(t) {
            let inbreeding = measure_inbreeding(&shards, k, nb_measurements, &mp, &sty, rng);
            println!("{}", inbreeding);
        }

        // recode a new random shard
        shards.shuffle(rng);
        let s: Vec<_> = shards.iter().take(nb_shards_to_recode).cloned().collect();
        let new_shard = fec::recode_random(&s, rng).unwrap().unwrap();
        shards.push(new_shard);

        pb.inc(1);
    }
    pb.finish_with_message("done");

    Ok(())
}

#[derive(ValueEnum, Clone)]
enum TestCase {
    EndToEnd,
    Recoding,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg()]
    nb_bytes: usize,

    #[arg(short)]
    k: usize,
    #[arg(short)]
    n: usize,
    #[arg(short)]
    t: usize,
    #[arg(short)]
    r: Option<usize>,

    #[arg(long)]
    test_case: TestCase,

    /// the number of measurements to repeat each case, larger values will reduce the variance of
    /// the measurements
    #[arg(long)]
    nb_measurements: usize,

    #[arg(long)]
    measurement_schedule: usize,
}

fn main() {
    let cli = Cli::parse();

    if cli.nb_measurements == 0 {
        eprintln!(
            "`--nb-measurements` should be a strictly positive integer, found {}",
            cli.nb_measurements
        );
        exit(1);
    }

    let mut rng = thread_rng();

    let bytes = random_bytes(cli.nb_bytes, &mut rng);

    eprintln!(
        "diversity will be measured every {} steps",
        cli.measurement_schedule
    );
    let measurement_schedule = |t| t % cli.measurement_schedule == 0;

    match cli.test_case {
        TestCase::EndToEnd => {
            let _ = end_to_end::<ark_pallas::Fr, _>(
                &bytes,
                cli.k,
                cli.n,
                cli.t,
                cli.nb_measurements,
                measurement_schedule,
                &mut rng,
            );
        }
        TestCase::Recoding => {
            if cli.r.is_none() {
                eprintln!("recoding needs -r");
                exit(1);
            }

            let _ = recoding::<ark_pallas::Fr, _>(
                &bytes,
                cli.k,
                cli.n,
                cli.t,
                cli.r.unwrap(),
                cli.nb_measurements,
                measurement_schedule,
                &mut rng,
            );
        }
    }
}
