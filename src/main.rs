mod app;
mod cpu;
mod font;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = format!("/home/martin/Projects/chreight/{}", args[1]);
    println!("{:?}", path);
    app::start(&path);

    // let mut c = CPU::new();
    // c.load_rom("/home/martin/Downloads/chip8games/PONG");
    // loop {
    //     sleep(Duration::new(1 / HERTZ, 0));
    //     c.execute_cycle();
    // }
}
