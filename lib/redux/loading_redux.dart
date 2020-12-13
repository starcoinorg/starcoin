import 'package:starcoin_node/config/actions.dart';
import 'package:redux/redux.dart';

final rLoadingReducer = combineReducers<bool>([
  TypedReducer<bool, LoadingAction>(_toggle),
]);

bool _toggle(bool visible, LoadingAction action) {
  print('$visible   |  $action');
  return visible == action.needLoadingVisible ? visible : !visible;
}
