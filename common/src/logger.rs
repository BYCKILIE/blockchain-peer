use std::{
    collections::hash_map::DefaultHasher,
    env,
    hash::{Hash, Hasher},
};
use tokio::{
    fs::OpenOptions,
    io::{AsyncWriteExt, BufWriter as TokioBufWriter},
    sync::{Mutex, OnceCell},
};

pub struct Logger;

impl Logger {
    fn color_code_for(input: &str) -> u8 {
        const COLORS: [u8; 6] = [31, 32, 33, 34, 35, 36];
        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        COLORS[(hasher.finish() as usize) % COLORS.len()]
    }

    pub fn console(from: &str, message: &str) {
        let code = Self::color_code_for(from);
        println!(
            "\x1b[{}m[{}]\x1b[0m {}",
            code,
            from.to_uppercase(),
            message
        );
    }
    
    pub async fn file(from: &str, message: &str) {
        let writer = LOG_WRITER
            .get_or_init(|| async {
                let path = env::var("LOG_PATH").unwrap_or_else(|_| "app.log".into());
                let file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)
                    .await
                    .expect("failed to open log file");
                Mutex::new(TokioBufWriter::new(file))
            })
            .await;

        let mut guard = writer.lock().await;
        let entry = format!("[{}] {}\n", from.to_uppercase(), message);
        let _ = guard.write_all(entry.as_bytes()).await;
        let _ = guard.flush().await;
        
        drop(guard);
    }
}

static LOG_WRITER: OnceCell<Mutex<TokioBufWriter<tokio::fs::File>>> =
    OnceCell::const_new();
