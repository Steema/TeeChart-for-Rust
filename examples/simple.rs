use teechart::Chart;

fn main() -> Result<(), teechart::Error> {
    Chart::new()
        .y(&[2.5, 3.2, 2.8, 4.1])
        .texts(&["One", "Two", "Three", "Four"])
        .style("pie")
        .title("TeeChart for Rust")
        .render()
}
