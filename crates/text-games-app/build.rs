fn main() {
    ink_dioxus::build::generate_ink_game_catalog("assets/ink")
        .expect("ink game catalog should generate");
}
