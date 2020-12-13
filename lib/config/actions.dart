


enum Action {

  ChangeTheme,

  ToggleLoading,

}

class LoadingAction {

  Action action = Action.ToggleLoading;

  bool needLoadingVisible;

  LoadingAction(this.needLoadingVisible);

}