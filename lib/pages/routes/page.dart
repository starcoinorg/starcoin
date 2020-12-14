import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:starcoin_node/pages/intro_page.dart';
import 'dart:io';

import 'package:starcoin_node/pages/node_page.dart';

class Page {
  final String title;

  final IconData leadingIcon;

  final VoidCallback leadingEvent;

  final String routeName;

  final WidgetBuilder buildRoute;

  Page(
      {this.title,
      this.leadingIcon,
      this.leadingEvent,
      @required this.routeName,
      @required this.buildRoute});

  @override
  String toString() {
    return 'Page{routeName: $routeName, buildRoute: $buildRoute}';
  }
}

final List<Page> kAllPages = _buildPages();

List<Page> _buildPages() {
  final List<Page> pages = <Page>[
    new Page(
        routeName: IntroPage.routeName,
        buildRoute: (BuildContext context) => new IntroPage()),
  ];
  return pages;
}
