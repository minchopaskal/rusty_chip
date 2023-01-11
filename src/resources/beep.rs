use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct BeepResource {
    pub sound : Box<Handle<AudioSource>>,
}
