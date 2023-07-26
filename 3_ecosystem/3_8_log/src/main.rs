use std::{fmt, fs::OpenOptions, io, path::Path, str};

use dairy::Cow;
use serde::{ser::SerializeMap, Serializer};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tracing::{event, Event, Level, Subscriber};
use tracing_serde::AsSerde;
use tracing_subscriber::{
    fmt::{format::Writer, writer::Tee, FmtContext, FormatEvent, FormatFields},
    prelude::*,
    registry::LookupSpan,
};

struct TheJsonFormat<'a> {
    file: Cow<'a, Path>,
}

impl<'a> TheJsonFormat<'a> {
    fn new(file: impl Into<Cow<'a, Path>>) -> Self {
        Self { file: file.into() }
    }
}

fn serialize_event<S: Serializer>(
    ser: S,
    event: &Event,
    time_str: &str,
    file: &Path,
) -> Result<(), S::Error> {
    let mut ser_map = ser.serialize_map(None)?;

    ser_map.serialize_entry("time", time_str)?;
    ser_map.serialize_entry("file", file)?;

    let meta = event.metadata();
    ser_map.serialize_entry("lvl", &meta.level().as_serde())?;

    let mut visitor = tracing_serde::SerdeMapVisitor::new(ser_map);
    event.record(&mut visitor);
    ser_map = visitor.take_serializer()?;

    ser_map.end()?;

    Ok(())
}

impl<'a, S, N> FormatEvent<S, N> for TheJsonFormat<'a>
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let time_str = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .map_err(|_| fmt::Error)?;

        let mut out: Vec<u8> = vec![];
        let mut ser = serde_json::Serializer::new(&mut out);
        serialize_event(&mut ser, event, &time_str, &self.file).map_err(|_| fmt::Error)?;

        let out_str = str::from_utf8(&out).map_err(|_| fmt::Error)?;

        writer.write_str(out_str)?;
        writeln!(writer)
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_writer(Tee::new(io::stdout, io::stderr.with_min_level(Level::WARN)))
        .event_format(TheJsonFormat::new("app.log"))
        .finish()
        .init();

    event!(Level::INFO, msg = "info", a = 1);
    event!(Level::ERROR, msg = "error", b = 2);

    tracing::subscriber::with_default(
        tracing_subscriber::fmt()
            .event_format(TheJsonFormat::new("access.log"))
            .with_writer(
                OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("access.log")
                    .unwrap(),
            )
            .finish(),
        || {
            event!(Level::ERROR, msg = "error2", c = 3);
        },
    );
}
