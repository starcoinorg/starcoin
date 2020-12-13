import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'dart:io';
import 'dart:convert';
import 'routes/routes.dart';

class MainPage extends StatefulWidget {
  static const String routeName = Routes.main + "/index";

  @override
  State createState() {
    return new _MainPageState();
  }
}

class _MainPageState extends State<MainPage> with TickerProviderStateMixin {
  Process process;
  String text = "";
  int maxLines = 10;
  List<String> lines = List();

  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final double iconSize = 60.0;
    final blue = Color.fromARGB(255, 0, 255, 255);
    final blueTextstyle = TextStyle(color: blue, fontSize: 30);
    final whiteTextstyle = TextStyle(color: Colors.white, fontSize: 30);
    final edgeTexts = EdgeInsets.only(left: 30, right: 30);
    final boxDecoration = new BoxDecoration(
      //设置四周圆角 角度
      borderRadius: BorderRadius.all(Radius.circular(10.0)),
      //设置四周边框
      border: new Border.all(width: 1, color: blue),
    );
    return Scaffold(
        body: Container(
            decoration: BoxDecoration(
                image: DecorationImage(
                    image: AssetImage("assets/images/starcoin-bg.png"),
                    fit: BoxFit.cover)),
            child: Container(
                margin: EdgeInsets.all(20),
                decoration: new BoxDecoration(
                  //设置四周圆角 角度
                  borderRadius: BorderRadius.all(Radius.circular(50.0)),
                  //设置四周边框
                  border: new Border.all(
                      width: 1, color: Color.fromARGB(120, 0, 255, 255)),
                ),
                child: Column(children: <Widget>[
                  Row(
                    children: <Widget>[
                      Image.asset(
                        'assets/images/starcoin-logo-fonts.png',
                        width: 200,
                        height: 100,
                      ),
                      Image.asset(
                        'assets/images/starcoin-miner.png',
                        width: 50,
                        height: 50,
                      ),
                      Text(
                        "星际争霸 第一期",
                        style: TextStyle(color: Colors.white, fontSize: 30),
                      ),
                    ],
                  ),
                  Container(
                    margin: EdgeInsets.only(left: 160, top: 10, right: 150),
                    alignment: Alignment(0, 0),
                    child: Center(
                        child: Column(children: <Widget>[
                      Container(
                          padding: EdgeInsets.only(
                              left: 20, right: 20, top: 10, bottom: 10),
                          decoration: new BoxDecoration(
                            color: blue,
                            //设置四周圆角 角度
                            borderRadius:
                                BorderRadius.all(Radius.circular(10.0)),
                            //设置四周边框
                            border: new Border.all(width: 1, color: blue),
                          ),
                          child: Row(
                            children: <Widget>[
                              Text(
                                "板板",
                                style: TextStyle(fontSize: 30),
                              ),
                              Container(
                                  padding: edgeTexts,
                                  child: Text(
                                      "0x0fb6d936ddc01ecb151d73d43c545251"))
                            ],
                          )),
                      SizedBox(height: 5),
                      Container(
                          padding: EdgeInsets.all(5),
                          decoration: boxDecoration,
                          child: Row(
                            children: <Widget>[
                              Image.asset(
                                'assets/images/starcoin-balance.png',
                                width: iconSize,
                                height: iconSize,
                              ),
                              Text(
                                "当前余额",
                                style: blueTextstyle,
                              ),
                              Container(
                                  padding: edgeTexts,
                                  child: Text("278", style: whiteTextstyle)),
                              Text("STC", style: blueTextstyle)
                            ],
                          )),
                      SizedBox(height: 5),
                      Container(
                          padding: EdgeInsets.all(5),
                          decoration: boxDecoration,
                          child: Row(
                            children: <Widget>[
                              Image.asset(
                                'assets/images/starcoin-block-number.png',
                                width: iconSize,
                                height: iconSize,
                              ),
                              Text(
                                "已挖块数",
                                style: blueTextstyle,
                              ),
                              Container(
                                  padding: edgeTexts,
                                  child: Text("27", style: whiteTextstyle)),
                              Text("块", style: blueTextstyle)
                            ],
                          )),
                      SizedBox(height: 5),
                      Container(
                          padding: EdgeInsets.all(5),
                          decoration: boxDecoration,
                          child: Row(
                            children: <Widget>[
                              Image.asset(
                                'assets/images/starcoin-difficulty.png',
                                width: iconSize,
                                height: iconSize,
                              ),
                              Text("当前难度", style: blueTextstyle),
                              Container(
                                  padding: edgeTexts,
                                  child: Text("2981", style: whiteTextstyle))
                            ],
                          )),
                      Row(
                        mainAxisAlignment: MainAxisAlignment.center,
                        crossAxisAlignment: CrossAxisAlignment.center,
                        children: <Widget>[
                          RaisedButton(
                            child: Text("Start Miner"),
                            onPressed: () async {
                              process = await Process.start(
                                  '/Users/fanngyuan/Documents/workspace/starcoin/target/debug/starcoin',
                                  ["-n", "dev", "--http-apis", "all"]);
                              process.stderr
                                  .transform(utf8.decoder)
                                  .listen((data) {
                                lines.add(data);
                                String tmpText;
                                if (lines.length < maxLines) {
                                  tmpText = lines.join();
                                } else {
                                  tmpText = lines
                                      .sublist(lines.length - maxLines)
                                      .join();
                                }
                                setState(() {
                                  text = tmpText;
                                });
                              });
                            },
                          ),
                          Container(
                            padding: EdgeInsets.all(10),
                            child: OutlineButton(
                              child: Text("Stop Miner"),
                              onPressed: () {
                                process.kill(ProcessSignal.sigterm);
                                setState(() {
                                  text = "";
                                });
                              },
                            ),
                          )
                        ],
                      ),
                      new Text(
                        text,
                        maxLines: maxLines,
                      ),
                    ])),
                  )
                ]))));
  }
}
