/// - build the example for best performance with [`./../../scripts/inbreeding/build.nu`]
/// - run the experiment with [`./../../scripts/inbreeding/run.nu`]
/// - plot the results with [`./../../scripts/inbreeding/plot.nu`]
///
/// # Example
/// ```nushell
/// use ./scripts/inbreeding
///
/// let opts = {
///     nb_bytes: (10 * 1_024),
///     k: 10,
///     n: 20,
///     nb_measurements: 10,
///     nb_scenarii: 100,
///     measurement_schedule: 1,
///     max_t: 150,
///     strategies: [
///         "single:1",
///         "double:0.5:1:2",
///         "single:2"
///         "double:0.5:2:3",
///         "single:3"
///         "single:5",
///         "single:10",
///     ],
///     environment: "fixed:0",
/// }
///
/// inbreeding build
/// inbreeding run --output data/inbreeding.nuon --options $opts
/// inbreeding plot data/inbreeding.nuon --options { k: $opts.k }
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

mod environment;
mod strategy;

use crate::{environment::Environment, strategy::Strategy};

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
            println!("{}, {}", t, inbreeding);
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
    strategy: Strategy,
    env: Environment,
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
        if shards.len() < k {
            break;
        }

        if measurement_schedule(t) {
            let inbreeding = measure_inbreeding(&shards, k, nb_measurements, &mp, &sty, rng);
            println!("{}, {}", t, inbreeding);
        }

        // recode a new random shard
        let new_shard = fec::recode_random(&strategy.draw(&shards, rng), rng)
            .unwrap()
            .unwrap();
        shards.push(new_shard);

        shards = env.update(&shards, rng);

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
    /// something of the form `<p>:<i>,<j>`
    /// at each time step, shard $i$ will be used for recoding with probability $p$, otherwise, $j$
    /// will be used with probability $1 - p$
    #[arg(long)]
    strategy: Option<String>,
    /// something of the form `random-dynamic:<p>:<q>` where a proportion $q$ of the shards will be removed at
    /// each step with probability $p$
    #[arg(long)]
    environment: Option<String>,

    #[arg(long)]
    test_case: TestCase,

    /// the number of measurements to repeat each case, larger values will reduce the variance of
    /// the measurements
    #[arg(long)]
    nb_measurements: usize,

    #[arg(long)]
    measurement_schedule: usize,
    #[arg(long)]
    measurement_schedule_start: usize,
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
    let measurement_schedule = |t| {
        t >= cli.measurement_schedule_start
            && (t - cli.measurement_schedule_start) % cli.measurement_schedule == 0
    };

    match cli.test_case {
        TestCase::EndToEnd => {
            eprintln!("naive: k = {}, n = {}", cli.k, cli.n);
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
            if cli.strategy.is_none() {
                eprintln!("recoding needs --strategy");
                exit(1);
            }
            if cli.environment.is_none() {
                eprintln!("recoding needs --environment");
                exit(1);
            }

            let environment = Environment::from_str(&cli.environment.unwrap()).unwrap();
            let strategy = Strategy::from_str(&cli.strategy.unwrap()).unwrap();

            eprintln!(
                "true: k = {}, n = {}, strategy = {:?}, environment = {:?}",
                cli.k, cli.n, strategy, environment,
            );
            let _ = recoding::<ark_pallas::Fr, _>(
                &bytes,
                cli.k,
                cli.n,
                cli.t,
                strategy,
                environment,
                cli.nb_measurements,
                measurement_schedule,
                &mut rng,
            );
        }
    }
}
