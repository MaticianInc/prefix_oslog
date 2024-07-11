use log::LevelFilter;
use oslog::OsLogger;

fn main() {
    OsLogger::level_filter(LevelFilter::Trace);

    OsLogger::new("com.example.test", LevelFilter::Info, true)
        .from_env()
        .init()
        .unwrap();

    log::trace!("Outer");
    log::debug!("Outer");
    log::info!("Outer");
    log::warn!("Outer");
    log::error!("Outer");

    foo::foo();
    foo::bar::bar();
    foo::baz::baz();
}

mod foo {
    pub fn foo() {
        log::trace!("foo");
        log::debug!("foo");
        log::info!("foo");
        log::warn!("foo");
        log::error!("foo");
    }
    pub mod bar {
        pub fn bar() {
            log::trace!("bar");
            log::debug!("bar");
            log::info!("bar");
            log::warn!("bar");
            log::error!("bar");
        }
    }

    pub mod baz {
        pub fn baz() {
            log::trace!("baz");
            log::debug!("baz");
            log::info!("baz");
            log::warn!("baz");
            log::error!("baz");
        }
    }
}
