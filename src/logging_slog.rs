use chrono;
use slog::{self, slog_o, Drain};
use slog_async;
use slog_json;
use std::env;

pub fn default_json_drain() -> impl Drain {
  let drain = slog_json::Json::new(std::io::stdout())
    .add_key_value(slog_o!(
        "msg" => slog::PushFnValue(move |record : &slog::Record, ser| {
            ser.emit(record.msg())
        }),
        "tag" => slog::PushFnValue(move |record : &slog::Record, ser| {
            ser.emit(record.tag())
        }),
        "ts" => slog::PushFnValue(move |_ : &slog::Record, ser| {
            ser.emit(chrono::Local::now().to_rfc3339())
        }),
        "level" => slog::FnValue(move |rinfo : &slog::Record| {
            rinfo.level().as_str()
        }),
     )).build()
    .fuse();
  let mut log_builder = slog_envlogger::LogBuilder::new(drain).filter(None, slog::FilterLevel::Info);
  if let Ok(s) = env::var("RUST_LOG") {
    log_builder = log_builder.parse(&s);
  }
  slog_async::Async::default(log_builder.build())
}
