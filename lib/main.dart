import 'package:flutter/material.dart';
import 'package:redux/redux.dart';
import 'package:flutter_redux/flutter_redux.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:starcoin_node/pages/localizations.dart';
import 'package:starcoin_node/pages/main_page.dart';
import 'package:starcoin_node/style/themes.dart';
import 'package:flutter_localizations/flutter_localizations.dart';

import 'config/states.dart';
import 'pages/routes/page.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  final sharedPrefs = await SharedPreferences.getInstance();
  final userName = sharedPrefs.getString("user_name");

  Store<AppState> store = new Store(appReducer,
      initialState: new AppState(theme: kLightTheme, loadingVisible: false));
  runApp(new App(
    store: store,
    userName: userName,
  ));
}

class App extends StatelessWidget {
  final Store<AppState> store;
  String userName;

  App({this.store, this.userName});

  @override
  Widget build(BuildContext context) {
    return StoreProvider(
        store: store,
        child: new StoreBuilder<AppState>(builder: (context, store) {
          bool needLoadingVisible = store.state.loadingVisible;
          return Stack(
            alignment: Alignment.center,
            children: <Widget>[
              Opacity(
                opacity: needLoadingVisible ? 1.0 : 0.0,
                child: _buildGlobalLoading(context),
              ),
              new MaterialApp(
                localizationsDelegates: [
                  // 本地化的代理类
                  GlobalMaterialLocalizations.delegate,
                  GlobalWidgetsLocalizations.delegate,
                  StarcoinLocalizationsDelegate(),
                ],
                supportedLocales: [
                  const Locale('en', 'US'), // 美国英语
                  const Locale('zh', 'CN'), // 中文简体
                  //其它Locales
                ],
                theme: store.state.theme.themeData,
                routes: _buildRoutes(),
                home: new MainPage(userName),
              ),
            ],
          );
        }));
  }

  Map<String, WidgetBuilder> _buildRoutes() {
    return new Map<String, WidgetBuilder>.fromIterable(
      kAllPages,
      key: (dynamic page) => '${page.routeName}',
      value: (dynamic page) => page.buildRoute,
    );
  }

  _buildGlobalLoading(BuildContext context) {
    final ThemeData theme = Theme.of(context);
    return Stack(
      alignment: Alignment.center,
      children: <Widget>[
        const ModalBarrier(
          color: Colors.grey,
        ),
        Container(
          width: 102.0,
          height: 102.0,
          padding: EdgeInsets.all(24.0),
          decoration: BoxDecoration(
              color: theme.backgroundColor,
              borderRadius: BorderRadius.all(Radius.circular(12.0))),
          child: new CircularProgressIndicator(),
        )
      ],
    );
  }
}
