// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::MyWorld;
use cucumber::steps;

steps!(MyWorld => {
    when regex "^a project with slug \"(.*)\" is submitted$" |world, matches, _| {

    };

    then regex "^a project with slug \"(.*)\" is created$" |world, matches, _| {

    };
});
