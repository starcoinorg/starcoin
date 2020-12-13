import 'package:starcoin_node/redux/loading_redux.dart';
import 'package:starcoin_node/redux/theme_redux.dart';
import 'package:starcoin_node/style/themes.dart';

class AppState {
  final LTheme theme;

  final bool loadingVisible;
  AppState({this.theme, this.loadingVisible = false});
}

AppState appReducer(AppState state, action) {
  return AppState(
    theme: rThemeDataReducer(state.theme, action),
    loadingVisible: rLoadingReducer(state.loadingVisible, action),
  );
}
