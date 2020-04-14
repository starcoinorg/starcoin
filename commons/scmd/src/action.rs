// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{Command, ExecContext};
use anyhow::Result;
use std::marker::PhantomData;
use structopt::StructOpt;

#[derive(Debug, Clone, Default, StructOpt)]
pub struct EmptyOpt {}

pub trait CommandAction {
    type State;
    type GlobalOpt: StructOpt;
    type Opt: StructOpt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()>;

    fn into_cmd(self) -> Command<Self::State, Self::GlobalOpt, Self::Opt, Self>
    where
        Self: std::marker::Sized,
    {
        self.into()
    }
}

pub struct FnCommandAction<State, GlobalOpt, Opt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
    Opt: StructOpt + 'static,
{
    action: Box<dyn Fn(&ExecContext<State, GlobalOpt, Opt>) -> Result<()>>,
}

impl<State, GlobalOpt, Opt> FnCommandAction<State, GlobalOpt, Opt>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
{
    pub fn new<A>(action: A) -> Self
    where
        A: Fn(&ExecContext<State, GlobalOpt, Opt>) -> Result<()> + 'static,
    {
        Self {
            action: Box::new(action),
        }
    }
}

impl<State, GlobalOpt, Opt> CommandAction for FnCommandAction<State, GlobalOpt, Opt>
where
    GlobalOpt: StructOpt,
    Opt: StructOpt,
{
    type State = State;
    type GlobalOpt = GlobalOpt;
    type Opt = Opt;

    fn run(&self, ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        self.action.as_ref()(ctx)
    }
}

pub struct NoneAction<State, GlobalOpt>
where
    State: 'static,
    GlobalOpt: StructOpt + 'static,
{
    state_type: PhantomData<State>,
    global_opt_type: PhantomData<GlobalOpt>,
}

impl<State, GlobalOpt> CommandAction for NoneAction<State, GlobalOpt>
where
    GlobalOpt: StructOpt,
{
    type State = State;
    type GlobalOpt = GlobalOpt;
    type Opt = EmptyOpt;

    fn run(&self, _ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>) -> Result<()> {
        Ok(())
    }
}
