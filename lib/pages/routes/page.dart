import 'package:flutter/material.dart';
import 'package:flutter_hooks/flutter_hooks.dart';
import 'dart:io';

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
  final List<Page> pages = <Page>[];
  return pages;
}
