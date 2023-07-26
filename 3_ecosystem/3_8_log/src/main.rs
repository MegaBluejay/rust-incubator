use core::fmt;
use std::{fs::OpenOptions, io, str};

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
    ) -> std::fmt::Result {
        let timestamp = OffsetDateTime::now_utc().format(&Rfc3339).unwrap();
        let meta = event.metadata();

        let do_ser = || {
            let mut out: Vec<u8> = vec![];
            let mut ser = serde_json::Serializer::new(&mut out);
            let mut ser = ser.serialize_map(None)?;

            ser.serialize_entry("time", &timestamp)?;
            ser.serialize_entry("lvl", &meta.level().as_serde())?;

            let mut visitor = tracing_serde::SerdeMapVisitor::new(ser);
            event.record(&mut visitor);
            ser = visitor.take_serializer()?;

            ser.end()?;
            Ok(out)
        };
        let out = do_ser().map_err(
            |_: <&mut serde_json::Serializer<&mut Vec<u8>> as Serializer>::Error| fmt::Error,
        )?;
        writer.write_str(str::from_utf8(&out).map_err(|_| fmt::Error)?)?;
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
