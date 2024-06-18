use crate::application::Application;

pub trait Plugin {
    fn build(&mut self, app: &mut Application);
}
