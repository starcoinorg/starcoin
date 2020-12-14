import 'package:flutter/material.dart';
import 'package:starcoin_node/pages/intro_page.dart';
import 'package:starcoin_node/pages/node_page.dart';
import 'package:starcoin_node/pages/routes/routes.dart';

class MainPage extends StatelessWidget {
  static const String routeName = Routes.main + "/main";

  final String userName;

  MainPage(this.userName);

  @override
  Widget build(BuildContext context) {
    if (userName != null) {
      return NodePage(userName);
    } else {
      return IntroPage();
    }
  }
}
