use bevy::{ecs::component::ComponentId, prelude::*};

pub mod prelude {
    pub use super::RegisterMutuallyExclusiveComponent;
}

pub trait RegisterMutuallyExclusiveComponent {
    fn register_mutually_exclusive_component<const GROUP: u32, C: Component>(
        &mut self,
    ) -> &mut Self;
}

impl RegisterMutuallyExclusiveComponent for App {
    fn register_mutually_exclusive_component<const GROUP: u32, C: Component>(
        &mut self,
    ) -> &mut Self {
        self.world_mut()
            .register_component_hooks::<C>()
            // `on_add` will trigger when a component is inserted onto an entity without it
            .on_add(|mut world, entity, component_id| {
                let mut to_remove = None;
                if let Some(mut last_mutex) =
                    world.get_mut::<LastMutuallyExclusiveId<GROUP>>(entity)
                {
                    to_remove = Some(last_mutex.0);
                    last_mutex.0 = component_id;
                } else {
                    world
                        .commands()
                        .entity(entity)
                        .insert(LastMutuallyExclusiveId::<GROUP>(component_id));
                }
                if let Some(old_component_id) = to_remove {
                    // remove existing mutex component:
                    world
                        .commands()
                        .entity(entity)
                        .remove_by_id(old_component_id);
                }
            })
            // `on_remove` will trigger when a component is removed from an entity,
            // since it runs before the component is removed you can still access the component data
            .on_remove(|mut world, entity, component_id| {
                if let Some(last_mutex) = world.get::<LastMutuallyExclusiveId<GROUP>>(entity) {
                    if last_mutex.0 == component_id {
                        // no other mutex component was inserted to trigger our removal, or their
                        // component id would be stored here. So we must be the last mutex component left on the entity.
                        // so we can cleanup after ourselves by removing this:
                        world
                            .commands()
                            .entity(entity)
                            .remove::<LastMutuallyExclusiveId<GROUP>>();
                    }
                }
            });
        self
    }
}

#[derive(Component)]
struct LastMutuallyExclusiveId<const GROUP: u32>(ComponentId);

mod test {
    use super::*;
    #[derive(Component)]
    struct A;
    #[derive(Component)]
    struct B;
    #[derive(Component)]
    struct C;

    #[derive(Component)]
    struct X;
    #[derive(Component)]
    struct Y;

    #[test]
    fn test_mutex_comps() {
        let mut app = App::new();
        const G: u32 = 1;
        const OTHER_G: u32 = 2;
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::log::LogPlugin::default());
        app.register_mutually_exclusive_component::<G, A>();
        app.register_mutually_exclusive_component::<G, B>();
        app.register_mutually_exclusive_component::<G, C>();
        app.register_mutually_exclusive_component::<OTHER_G, X>();
        app.register_mutually_exclusive_component::<OTHER_G, Y>();
        app.update();
        let e1 = app.world_mut().spawn(A).id();
        app.update();
        app.world_mut().commands().entity(e1).log_components();
        assert!(
            app.world().get::<LastMutuallyExclusiveId<G>>(e1).is_some(),
            "e1 should have a LastMutexComp"
        );
        assert_eq!(
            app.world().get::<LastMutuallyExclusiveId<G>>(e1).unwrap().0,
            app.world()
                .component_id::<A>()
                .expect("A should be registered"),
            "e1's LastMutexComp compoment it should match A"
        );
        app.update();
        app.world_mut().commands().entity(e1).insert(B);
        app.update();
        assert!(
            app.world().get::<A>(e1).is_none(),
            "A should have been removed"
        );
        assert!(app.world().get::<B>(e1).is_some(), "A should be inserted");
        assert_eq!(
            app.world().get::<LastMutuallyExclusiveId<G>>(e1).unwrap().0,
            app.world()
                .component_id::<B>()
                .expect("B should be registered"),
            "e1's LastMutexComp compoment it should match B"
        );
        app.world_mut().commands().entity(e1).log_components();
        app.update();
        app.world_mut().commands().entity(e1).insert((C, A));
        app.update();
        app.world_mut().commands().entity(e1).log_components();
        assert!(
            app.world().get::<B>(e1).is_none(),
            "B should have been removed"
        );
        assert!(
            app.world().get::<C>(e1).is_none(),
            "C should have been removed"
        );
        assert!(app.world().get::<A>(e1).is_some(), "A should be inserted");
        app.update();
        app.world_mut().commands().entity(e1).remove::<A>();
        app.update();
        app.world_mut().commands().entity(e1).log_components();
        app.update();
        assert!(
            app.world().get::<LastMutuallyExclusiveId<G>>(e1).is_none(),
            "e1 should not have a LastMutexComp"
        );
    }
}
