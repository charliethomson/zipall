use std::{collections::HashMap, path::PathBuf, time::Duration};

use clap::Parser;
use humansize::DECIMAL;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rand::Rng;
use tokio::task::JoinSet;
use zipall_core::{Scanner, ZipAllError, ZipAllResult, ZipMode, ZipSpecification, ZipStat, Zipper};
use zipall_log::setup_logger;

#[derive(Debug, Clone, Parser)]
struct Args {
    #[arg(default_value = ".")]
    pub src: String,

    #[arg(short, long, default_value = "./_Archives")]
    pub dest: String,

    #[arg(long, default_value_t = false)]
    pub dry: bool,

    #[arg(short, long, default_value = "7z")]
    pub bin: String,
}
#[tokio::main]
async fn main() -> ZipAllResult<()> {
    let args = Args::parse();

    setup_logger("zipall")?;

    let dest = PathBuf::from(args.dest);
    if !dest.exists() {
        tokio::fs::create_dir_all(&dest)
            .await
            .map_err(|e| ZipAllError::FailedToCreateDirectory(e.to_string()))?;
    }

    let dest = tokio::fs::canonicalize(dest).await.unwrap();
    let scanner = Scanner::new(&args.src, &dest).await?;

    let files = scanner.scan().await?;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<ZipStat>();

    let mut set = JoinSet::new();

    for (i, file) in files.iter().enumerate() {
        let ftx = tx.clone();
        let spec = ZipSpecification::new(&file, &dest, ZipMode::SevenZed)?;
        let mut zipper = Zipper::new(spec, i, PathBuf::from(&args.bin), ftx);

        set.spawn(async move { zipper.run().await });
    }

    let mb = MultiProgress::with_draw_target(ProgressDrawTarget::stdout());
    let sty = ProgressStyle::with_template(
        "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
    )
    .unwrap()
    .progress_chars("##-");

    // let mut rng = rand::thread_rng();
    // for i in 0..10 {
    //     let bar = mb.insert(i, ProgressBar::new(100));
    //     let dur = Duration::from_millis(rng.gen_range(1000..5000));
    //     set.spawn(async move {
    //         for _ in 0..100 {
    //             bar.inc(1);

    //             tokio::time::sleep(dur).await;
    //         }

    //         bar.finish();
    //     });
    // }

    let mut bars: HashMap<usize, ProgressBar> = HashMap::new();
    let mut get_bar = |id: usize| {
        let def = ProgressBar::new(100);
        def.set_style(sty.clone());

        if let Some(bar) = bars.get(&id) {
            return bar.downgrade();
        }

        let bar = mb.insert(id, def);
        let r = bar.downgrade();
        bars.insert(id, bar);
        return r;
    };

    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        tokio::select! {
            // _ = interval.tick() => {
            //     mb.println("Zipping shit").unwrap();
            //     }
            Some(stat) = rx.recv() => {
                match stat {
                    ZipStat::KeepAlive(id) => log::info!("[{id}] keep-alive"),
                    ZipStat::Progress { id, percent, filename } => {
                        let bar = get_bar(id).upgrade().unwrap();
                        bar.set_position(percent as u64);
                        bar.set_message(filename);
                    }
                }
            },
            result = set.join_next() => {
                match result {
                    Some(Ok(Ok(id)))  => {
                        log::info!("[{id}] Done");
                        let bar = get_bar(id);
                        bar.upgrade().unwrap().finish();
                    },
                    Some(Ok(Err(e)))  => {
                        log::warn!("Failure! {:#?}", e);
                    },
                    Some(Err(e))  => {
                        log::warn!("Join error! {:#?}", e);
                    },
                    None  => {
                        log::info!("All done!");
                        break
                    }
                }
            }
        };
    }

    Ok(())
}
