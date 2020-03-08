use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tracing::Subscriber;
use tracing::{event, span, Event, Level};
use tracing_layer_dispatch::DispatchLayer;
use tracing_subscriber::layer::{Context, SubscriberExt};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, Layer, Registry};

struct NoopLayer;

impl<S> Layer<S> for NoopLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let _event = black_box(event);
    }

    fn on_enter(&self, id: &span::Id, _ctx: Context<'_, S>) {
        let _id = black_box(id);
    }

    fn on_exit(&self, id: &span::Id, _ctx: Context<'_, S>) {
        let _id = black_box(id);
    }
}

/// Subscriber wrapped with layers
fn single() -> impl Subscriber {
    let log_filter_layer = EnvFilter::try_new("info").unwrap();

    Registry::default().with(log_filter_layer).with(NoopLayer)
}

/// Subscriber wrapped with a dispatch layer composed of single layer stack
fn dispatch_single() -> impl Subscriber {
    let layers = EnvFilter::try_new("info").unwrap().and_then(NoopLayer);
    Registry::default().with(DispatchLayer::new().with(layers))
}

/// Subscriber wrapped with a dispatch layer composed of single two layer stacks
fn dispatch_two() -> impl Subscriber {
    let layers_first = EnvFilter::try_new("info").unwrap().and_then(NoopLayer);
    let layers_second = EnvFilter::try_new("info").unwrap().and_then(NoopLayer);

    Registry::default().with(DispatchLayer::new().with(layers_first).with(layers_second))
}

fn run_event_in_span() {
    let span = span!(Level::INFO, "some_span");
    let _enter = span.enter();
    event!(Level::INFO, "something happened in span");
}

fn run_multiple_events_in_span() {
    let span = span!(Level::INFO, "some_span");
    let _enter = span.enter();
    event!(Level::TRACE, "trace in span");
    event!(Level::DEBUG, "debug in span");
    event!(Level::INFO, "info in span");
    event!(Level::WARN, "warn in span");
    event!(Level::ERROR, "error in span");
}

fn run_event_in_many_spans() {
    let span = span!(Level::TRACE, "trace_span");
    let _enter = span.enter();
    let span = span!(Level::DEBUG, "debug_span");
    let _enter = span.enter();
    let span = span!(Level::INFO, "info_span");
    let _enter = span.enter();
    let span = span!(Level::WARN, "warn_span");
    let _enter = span.enter();
    let span = span!(Level::ERROR, "error_span");
    let _enter = span.enter();
    event!(Level::INFO, "something happened in span");
}

fn single_event_in_span(c: &mut Criterion) {
    let mut group = c.benchmark_group("Single Event in Span");
    tracing::subscriber::with_default(single(), || {
        group.bench_function("Single layer stack", |b| b.iter(|| run_event_in_span()));
    });

    tracing::subscriber::with_default(dispatch_single(), || {
        group.bench_function("Dispatch single stack", |b| b.iter(|| run_event_in_span()));
    });

    tracing::subscriber::with_default(dispatch_two(), || {
        group.bench_function("Dispatch two stacks", |b| b.iter(|| run_event_in_span()));
    });
    group.finish()
}

fn multiple_events_in_span(c: &mut Criterion) {
    let mut group = c.benchmark_group("Multiple Events in Span");
    tracing::subscriber::with_default(single(), || {
        group.bench_function("Single layer stack", |b| {
            b.iter(|| run_multiple_events_in_span())
        });
    });

    tracing::subscriber::with_default(dispatch_single(), || {
        group.bench_function("Dispatch single stack", |b| {
            b.iter(|| run_multiple_events_in_span())
        });
    });

    tracing::subscriber::with_default(dispatch_two(), || {
        group.bench_function("Dispatch two stacks", |b| {
            b.iter(|| run_multiple_events_in_span())
        });
    });
    group.finish()
}

fn single_event_in_multiple_spans(c: &mut Criterion) {
    let mut group = c.benchmark_group("Single Event in multiple Spans");
    tracing::subscriber::with_default(single(), || {
        group.bench_function("Single layer stack", |b| {
            b.iter(|| run_event_in_many_spans())
        });
    });

    tracing::subscriber::with_default(dispatch_single(), || {
        group.bench_function("Dispatch single stack", |b| {
            b.iter(|| run_event_in_many_spans())
        });
    });

    tracing::subscriber::with_default(dispatch_two(), || {
        group.bench_function("Dispatch two stacks", |b| {
            b.iter(|| run_event_in_many_spans())
        });
    });
    group.finish()
}

criterion_group!(
    benches,
    single_event_in_span,
    multiple_events_in_span,
    single_event_in_multiple_spans
);
criterion_main!(benches);
