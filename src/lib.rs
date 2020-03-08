use tracing::subscriber::Interest;
use tracing::{span, Event, Metadata, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;

/// Allows multiple layers coexist in parallel. If one layer is not interested it won't affect
/// other layers managed by this dispatch. On the contrary, in standard layered structure, any
/// layer giving enabled=false or interest=never will suppress events and spans from being recorded
/// globally for all layers.
pub struct DispatchLayer<S>(Vec<Box<dyn Layer<S> + Send + Sync + 'static>>)
where
    S: Subscriber + for<'a> LookupSpan<'a>;

impl<S> DispatchLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    pub fn new() -> Self {
        DispatchLayer(Vec::new())
    }

    pub fn with<L: Layer<S> + Send + Sync + 'static>(mut self, layer_stack: L) -> Self {
        self.0.push(Box::new(layer_stack));

        self
    }
}

impl<S> Layer<S> for DispatchLayer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn register_callsite(&self, metadata: &'static Metadata<'static>) -> Interest {
        self.0.iter().map(|f| f.register_callsite(metadata)).fold(
            Interest::never(),
            |overall, particular| {
                if particular.is_always() {
                    particular
                } else if particular.is_sometimes() && !overall.is_always() {
                    particular
                } else {
                    overall
                }
            },
        )
    }

    fn enabled(&self, metadata: &Metadata<'_>, ctx: Context<'_, S>) -> bool {
        self.0.iter().any(|f| f.enabled(metadata, ctx.clone()))
    }

    fn new_span(&self, attrs: &span::Attributes<'_>, id: &span::Id, ctx: Context<'_, S>) {
        self.0
            .iter()
            .for_each(|inner| inner.new_span(attrs, id, ctx.clone()))
    }

    fn on_record(&self, span: &span::Id, values: &span::Record<'_>, ctx: Context<'_, S>) {
        self.0
            .iter()
            .for_each(|inner| inner.on_record(span, values, ctx.clone()))
    }

    fn on_follows_from(&self, span: &span::Id, follows: &span::Id, ctx: Context<'_, S>) {
        self.0
            .iter()
            .for_each(|inner| inner.on_follows_from(span, follows, ctx.clone()))
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        self.0
            .iter()
            .for_each(|inner| inner.on_event(event, ctx.clone()))
    }

    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        self.0
            .iter()
            .for_each(|inner| inner.on_enter(id, ctx.clone()))
    }

    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        self.0
            .iter()
            .for_each(|inner| inner.on_exit(id, ctx.clone()))
    }

    fn on_close(&self, id: span::Id, ctx: Context<'_, S>) {
        self.0
            .iter()
            .for_each(|inner| inner.on_close(id.clone(), ctx.clone()))
    }

    fn on_id_change(&self, old: &span::Id, new: &span::Id, ctx: Context<'_, S>) {
        self.0
            .iter()
            .for_each(|inner| inner.on_id_change(old, new, ctx.clone()))
    }
}
