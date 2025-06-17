#[macro_export]
macro_rules! make_enum_with_all_variants_array {
    (
        $name:ident $array:ident {
            $( $variant:ident, )*
        }
    ) => {
        #[derive(clap::ValueEnum, Clone, Hash, PartialEq, Eq, Debug)]
        pub enum $name {
            $( $variant, )*
        }
        pub static $array: &[$name] = &[
            $( $name::$variant, )*
        ];
    }
}

#[macro_export]
macro_rules! timeit_and_discard_output {
    ($( $body:stmt );* $(;)?) => {
        plnk::timeit(|| {
            let _ = { $( $body )* };
        })
    };
}

#[macro_export]
macro_rules! update_progress_bar_with_serializable_items {
    ($pb:ident : $( $items:ident ),* $(,)?) => {{
        use ark_serialize::CanonicalSerialize;
        let msg = $pb.message();
        let msg_tokens = msg.split(" ").collect::<Vec<_>>();
        let so_far = msg_tokens.last().unwrap().parse::<usize>().unwrap();

        let sizes: Vec<usize> = vec![
             $( $items.serialized_size(ark_serialize::Compress::No), )*
        ];
        let current: usize = sizes.into_iter().sum();
        let new = so_far + current;

        $pb.inc(1);
        $pb.set_message(format!(
            "{} {} {}",
            &msg_tokens[..msg_tokens.len() - 3].join(" "),
            $crate::pretty::filesize(new),
            new,
        ));
    }};
}
