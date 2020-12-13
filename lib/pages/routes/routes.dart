
import 'package:flutter/material.dart';

class Routes {

  static const main = '/main';

  static const market = '/market';

  static const wallet = '/wallet';

  static const profile = '/profile';


  /// 创建一个平移变换
  /// 跳转过去查看源代码，可以看到有各种各样定义好的变换
  static SlideTransition bottom2TopTransition(
      Animation<double> animation, Widget child) {
    return new SlideTransition(
      position: new Tween<Offset>(
        begin: const Offset(0.0, 1.0),
        end: const Offset(0.0, 0.0),
      ).animate(animation),
      child: child, // child is the value returned by pageBuilder
    );
  }


}
