use tracing::Subscriber;
use tracing_subscriber::fmt::MakeWriter;

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
    use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formmating_layer = BunyanFormattingLayer::new(name, sink);

    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formmating_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    use tracing::subscriber::set_global_default;
    use tracing_log::LogTracer;

    LogTracer::init().expect("Fail to set logger");
    set_global_default(subscriber).expect("Fail to set subscriber");
}
