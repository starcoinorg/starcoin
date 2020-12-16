import 'dart:async';

import 'package:flutter/services.dart';

class DirectoryService {
  //Method channel creation
  static const platform = const MethodChannel('wallet.starcoin.org/channel');
  //Adding the listener into contructor
  DeepLinkBloc() {}

  static Future<String> getCurrentDirectory() async {
    String result;
    try {
      result = await platform.invokeMethod('getCurrentDirectory');
    } on PlatformException catch (e) {
      rethrow;
    }
    return result;
  }
}
