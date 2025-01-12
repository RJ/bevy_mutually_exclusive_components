# Bevy Mutually Exclusive Components

Mutually exclusive groups of components for bevy.

## Usage

There is no plugin, just bring the `RegisterMutuallyExclusiveComponent` trait into scope:

```rust
use bevy_mutually_exclusive_components::prelude::*;
```

Components in a group (defined by a `u32`) are mutually exclusive – if you set up a group with
components `A`, `B`, and `C`, then insert `A` into an entity, then insert `B`, `A` will be removed.

```rust
#[derive(Component)]
struct A;
#[derive(Component)]
struct B;
#[derive(Component)]
struct C;

// ...

// Make components A,B,C mutually exclusive
const MY_GROUP: u32 = 1;
app.register_mutually_exclusive_component::<MY_GROUP, A>();
app.register_mutually_exclusive_component::<MY_GROUP, B>();
app.register_mutually_exclusive_component::<MY_GROUP, C>();

// Make components X,Y mutually exclusive
const MY_OTHER_GROUP: u32 = 2;
app.register_mutually_exclusive_component::<MY_OTHER_GROUP, X>();
app.register_mutually_exclusive_component::<MY_OTHER_GROUP, Y>();

```

## How it works

A component hook is created for each component you register. It stores the `ComponentId` of itself
in a private `LastMutuallyExclusiveId` component on insertion.

Before doing so, it checks if there is already a value in the `LastMutuallyExclusiveId` component,
and if so, removes the component with that `ComponentId`.

See the test at the bottom of `lib.rs` for an example.

The `LastMutuallyExclusiveId` component takes a const generic u32, which is how groups are identified.

## Caution

Components can only have one hook, so it'll panic if you try to add a component to multiple groups.

It will also panic if a component already has a hook for another reason.

## License

Same as bevy