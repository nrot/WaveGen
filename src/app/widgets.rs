

#[macro_export]
macro_rules! hseparator {
    ($ui:tt)=>{
        $ui.add(egui::Separator::default().horizontal());
    }
}

#[macro_export]
macro_rules! vseparator {
    ($ui:tt)=>{
        $ui.add(egui::Separator::default().vertical());
    }
}