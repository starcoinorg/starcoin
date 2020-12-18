//Locale资源类
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

class StarcoinLocalizations {
  StarcoinLocalizations(this.isZh);
  //是否为中文
  bool isZh = false;
  //为了使用方便，我们定义一个静态方法
  static StarcoinLocalizations of(BuildContext context) {
    return Localizations.of<StarcoinLocalizations>(
        context, StarcoinLocalizations);
  }

  //Locale相关值，title为应用标题
  String get title {
    return isZh ? "Starcoin 挖矿程序" : "Starcoin Miner";
  }

  String get slogon {
    return isZh ? "参与测试网挖矿 瓜分万U!" : "Start Mining Win10K USDT/STC!";
  }

  String get currentTask {
    return isZh ? "当前任务" : "Current Task";
  }

  String get progress {
    return isZh ? "进度" : "Progress";
  }

  String get balance {
    return isZh ? "当前余额" : "Balance";
  }

  String get minedBlocks {
    return isZh ? "已挖块数" : "Mined";
  }

  String get blockUnit {
    return isZh ? "块" : "Blocks";
  }

  String get currentDiff {
    return isZh ? "当前难度" : "Current Difficulty";
  }

  String get createNickyName {
    return isZh ? "创建昵称" : "Create Nicky Name";
  }

  String get confirm {
    return isZh ? "确认" : "Confirm";
  }

  String get generatePoster {
    return isZh ? "生成海报" : "Share Poster";
  }

  String get offcialWebSite {
    return isZh ? "官网" : "Offcial Website";
  }
}

class StarcoinLocalizationsDelegate
    extends LocalizationsDelegate<StarcoinLocalizations> {
  const StarcoinLocalizationsDelegate();

  //是否支持某个Local
  @override
  bool isSupported(Locale locale) => ['en', 'zh'].contains(locale.languageCode);

  // Flutter会调用此类加载相应的Locale资源类
  @override
  Future<StarcoinLocalizations> load(Locale locale) {
    print("$locale");
    return SynchronousFuture<StarcoinLocalizations>(
        StarcoinLocalizations(locale.languageCode == "zh"));
  }

  @override
  bool shouldReload(StarcoinLocalizationsDelegate old) => false;
}
