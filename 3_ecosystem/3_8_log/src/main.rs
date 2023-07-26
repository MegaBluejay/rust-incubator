use std::{fmt, fs::OpenOptions, io, str};

use serde::{ser::SerializeMap, Serializer};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tracing::{event, metadata::LevelFilter, Event, Level, Subscriber};
use tracing_serde::AsSerde;
use tracing_subscriber::{
    fmt::{format::Writer, FmtContext, FormatEvent, FormatFields},
    prelude::*,
    registry::LookupSpan,
};

struct TheJsonFormat;

fn serialize_event<S: Serializer>(ser: S, event: &Event, time_str: &str) -> Result<(), S::Error> {
    let mut ser_map = ser.serialize_map(None)?;

    ser_map.serialize_entry("time", time_str)?;

    let meta = event.metadata();
    ser_map.serialize_entry("lvl", &meta.level().as_serde())?;

    let mut visitor = tracing_serde::SerdeMapVisitor::new(ser_map);
    event.record(&mut visitor);
    ser_map = visitor.take_serializer()?;

    ser_map.end()?;

    Ok(())
}

impl<S, N> FormatEvent<S, N> for TheJsonFormat
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
        serialize_event(&mut ser, event, &time_str).map_err(|_| fmt::Error)?;

        let out_str = str::from_utf8(&out).map_err(|_| fmt::Error)?;

        writer.write_str(out_str)?;
        writeln!(writer)
    }
}

fn main() {
    let out_layer = tracing_subscriber::fmt::layer().event_format(TheJsonFormat);
    let err_layer = tracing_subscriber::fmt::layer()
        .with_writer(io::stderr)
        .event_format(TheJsonFormat)
        .with_filter(LevelFilter::WARN);

    tracing_subscriber::registry()
        .with(out_layer)
        .with(err_layer)
        .init();

    event!(Level::INFO, msg = "info", a = 1);
    event!(Level::ERROR, msg = "error", b = 2);

    tracing::subscriber::with_default(
        tracing_subscriber::fmt()
            .event_format(TheJsonFormat)
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
