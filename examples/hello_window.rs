use tx2_core::App;

fn main() {
    pollster::block_on(App::run());
}
