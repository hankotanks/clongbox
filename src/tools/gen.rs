use std::ops;

pub struct GenTool {
    prob_mono: f64,
    prob_dropoff: f64,
}

impl Default for GenTool {
    fn default() -> Self {
        Self {
            prob_mono: 2.5,
            prob_dropoff: 2.5,
        }
    }
}

impl super::Tool for GenTool {
    fn name(&self) -> &'static str { "Word Generation" }

    fn show(&mut self, _state: &mut crate::State, ui: &mut egui::Ui) {
        let Self {
            prob_mono,
            prob_dropoff, ..
        } = self;

        let prob_mono_slider = egui::Slider::new(
            prob_mono, 
            ops::RangeInclusive::new(0., 6.)
        ).custom_formatter(|n, _| {
            fn contains(start: f64, end: f64, n: f64) -> bool {
                ops::RangeInclusive::new(start, end).contains(&n)
            }

            let content = if contains(0., 1., n) {
                "Never"
            } else if contains(1., 2., n) {
                "Rare"
            } else if contains(2., 3., n) {
                "Less Frequent"
            } else if contains(3., 4., n) {
                "Frequent"
            } else if contains(4., 5., n) {
                "Mostly"
            } else if contains(5., 6., n) {
                "Always"
            } else {
                unreachable!();
            };
            
            // TODO: This allocation happens EVERY frame, should prevent
            String::from(content)
        });

        ui.label("Monosyllables");

        ui.add(prob_mono_slider);

        let prob_dropoff_slider = egui::Slider::new(
            prob_dropoff,
            ops::RangeInclusive::new(0., 4.)
        ).custom_formatter(|n, _| {
            // TODO: This is a duplicated helper function, move out of closure
            fn contains(start: f64, end: f64, n: f64) -> bool {
                ops::RangeInclusive::new(start, end).contains(&n)
            }

            let content = if contains(0., 1., n) {
                "Equiprobable"
            } else if contains(1., 2., n) {
                "Slow"
            } else if contains(2., 3., n) {
                "Medium"
            } else if contains(3., 4., n) {
                "Fast"
            } else {
                unreachable!();
            };

            // TODO: This allocation happens EVERY frame, should prevent
            String::from(content)
        });

        ui.label("Dropoff");

        ui.vertical_centered_justified(|ui| {
            ui.add(prob_dropoff_slider);
        });

        ui.separator();  
    }
}