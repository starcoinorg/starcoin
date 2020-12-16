import 'dart:async';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:starcoin_wallet/starcoin/starcoin.dart';
import 'package:starcoin_wallet/wallet/node.dart';
import 'dart:io';
import 'dart:convert';
import 'directory_service.dart';
import 'routes/routes.dart';
import 'package:date_format/date_format.dart';
import "package:path/path.dart" show join;
import 'package:image/image.dart' as img;

const LOCALURL = "http://localhost:9850";

class NodePage extends StatefulWidget {
  static const String routeName = Routes.main + "/index";

  String userName;

  NodePage(this.userName);

  @override
  State createState() {
    return new _NodePageState(this.userName);
  }
}

class _NodePageState extends State<NodePage> with TickerProviderStateMixin {
  Process process;
  String text = "";
  double balance = 0;
  String difficulty = "0";
  int blocks = 0;
  int maxLines = 5;
  String time = "";
  List<String> lines = List();
  String address = "0x0fb6d936ddc01ecb151d73d43c545251";

  String userName;

  GlobalKey previewContainer = new GlobalKey();

  _NodePageState(this.userName);

  bool startRequest = false;

  @override
  void initState() {
    super.initState();
  }

  @override
  Widget build(BuildContext context) {
    final double iconSize = 60.0;
    final double buttonIconSize = 40.0;
    final blue = Color.fromARGB(255, 0, 255, 255);

    final blueTextstyle = TextStyle(color: blue, fontSize: 25);
    final whiteTextstyle = TextStyle(color: Colors.white, fontSize: 25);
    final edgeTexts = EdgeInsets.only(left: 30, right: 30);
    final dateTime = DateTime.now();
    Directory current = Directory.current;
    time = current.path;
    //time = formatDate(dateTime, [yyyy, '/', mm, '/', dd, ' ', HH, ':', nn]);
    //freshTime();
    final boxDecoration = new BoxDecoration(
      //设置四周圆角 角度
      borderRadius: BorderRadius.all(Radius.circular(10.0)),
      //设置四周边框
      border: new Border.all(width: 1, color: blue),
    );
    var onclick;
    if (!startRequest) {
      onclick = () async {
        // 用Directory.current 也不对
        var command ="";
        if (Platform.isMacOS){
          final current = await DirectoryService.getCurrentDirectory();
          final dir = Directory.fromUri(Uri.parse(current)).parent;
           command = join(dir.path, 'starcoin/starcoin');
        }
        if (Platform.isWindows){
          Directory current = Directory.current;
          command= join(current.path,'starcoin/starcoin');
        }
        final process = await Process.start(command
            ,
            [
              "-n",
              "dev",
              "--http-apis",
              "all",
              "--disable-mint-empty-block",
              "false"
            ],
            runInShell: false);
        process.stderr.transform(utf8.decoder).listen((data) {
          lines.add(data);
          if (data.contains("Mint new block")) {
            blocks++;
          }
          String tmpText;
          if (lines.length < maxLines) {
            tmpText = lines.join();
          } else {
            tmpText = lines.sublist(lines.length - maxLines).join();
          }
          setState(() {
            text = tmpText;
          });
        });
        startRequest = true;
        await freshData();
      };
    }
    var startButton = RaisedButton(
      padding: EdgeInsets.only(top: 10.0, bottom: 10, left: 30, right: 30),
      color: blue,
      child: Image.asset(
        "assets/images/starcoin-start-mint.png",
        width: buttonIconSize,
        height: buttonIconSize,
      ),
      shape: RoundedRectangleBorder(
        borderRadius: new BorderRadius.circular(20.0),
        side: BorderSide(color: blue),
      ),
      onPressed: onclick,
    );

    var onStop;

    if (startRequest) {
      onStop = () {
        process.kill(ProcessSignal.sigterm);
        setState(() {
          text = "";
          blocks = 0;
          startRequest = false;
        });
      };
    }
    var stopButton = Container(
      padding: EdgeInsets.all(10),
      child: RaisedButton(
        color: blue,
        padding: EdgeInsets.only(top: 10.0, bottom: 10, left: 30, right: 30),
        //borderSide: new BorderSide(color: blue),
        child: Image.asset(
          "assets/images/starcoin-stop-mint.png",
          width: buttonIconSize,
          height: buttonIconSize,
        ),
        shape: RoundedRectangleBorder(
          borderRadius: new BorderRadius.circular(20.0),
          side: BorderSide(color: blue),
        ),
        onPressed: onStop,
      ),
    );
    final node = Node(LOCALURL);
    return RepaintBoundary(
        key: previewContainer,
        child: Scaffold(
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
                          ),
                          Image.asset(
                            'assets/images/starcoin-miner.png',
                            width: 50,
                          ),
                          Text(
                            "参与测试网挖矿 瓜分万U!",
                            style: TextStyle(color: Colors.white, fontSize: 20),
                          ),
                          Expanded(
                              flex: 1,
                              child: Container(
                                  margin: EdgeInsets.only(right: 20),
                                  alignment: Alignment.centerRight,
                                  child: IconButton(
                                    icon: Image.asset(
                                        'assets/images/starcoin-save.png'),
                                    iconSize: 60,
                                    onPressed: () async {
                                      await takescrshot();
                                    },
                                  ))),
                        ],
                      ),
                      Container(
                        margin: EdgeInsets.only(left: 160, top: 10, right: 150),
                        alignment: Alignment(0, 0),
                        child: Center(
                            child: Column(children: <Widget>[
                          Container(
                              margin: EdgeInsets.only(bottom: 10),
                              alignment: Alignment.centerRight,
                              child: Text(
                                time,
                                style: TextStyle(color: blue, fontSize: 15),
                              )),
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
                                    userName,
                                    style: TextStyle(fontSize: 25),
                                  ),
                                  Container(
                                      padding: edgeTexts, child: Text(address))
                                ],
                              )),
                          SizedBox(height: 5),
                          Container(
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
                                      child: Text("$balance",
                                          style: whiteTextstyle)),
                                  Text("STC", style: blueTextstyle)
                                ],
                              )),
                          SizedBox(height: 5),
                          Container(
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
                                      child: Text("$blocks",
                                          style: whiteTextstyle)),
                                  Text("块", style: blueTextstyle)
                                ],
                              )),
                          SizedBox(height: 5),
                          Container(
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
                                      child: Text(difficulty,
                                          style: whiteTextstyle))
                                ],
                              )),
                          SizedBox(height: 5),
                          Row(
                            mainAxisAlignment: MainAxisAlignment.center,
                            crossAxisAlignment: CrossAxisAlignment.center,
                            children: <Widget>[startButton, stopButton],
                          ),
                          new Text(
                            text,
                            style: TextStyle(color: Colors.white),
                            maxLines: maxLines,
                          ),
                        ])),
                      )
                    ])))));
  }

  void freshTime() {
    Timer.periodic(Duration(seconds: 5), (timer) async {
      final dateTime = DateTime.now();
      setState(() {
        time = formatDate(dateTime, [yyyy, '/', mm, '/', dd, ' ', HH, ':', nn]);
      });
    });
  }

  void freshData() async {
    await Future.delayed(Duration(seconds: 5));
    final node = Node(LOCALURL);
    Timer.periodic(Duration(seconds: 10), (timer) async {
      if (startRequest) {
        final account = await node.defaultAccount();
        final address =
            AccountAddress.fromJson(account['address'].replaceAll("0x", ""));
        final balance = await node.balanceOfStc(address);
        final nodeInfo = await node.nodeInfo();
        final totalDifficulty =
            nodeInfo['peer_info']['chain_info']['total_difficulty'];

        setState(() {
          this.address = address.toString();
          this.balance = balance.toBigInt() / BigInt.from(1000000000);
          this.difficulty = totalDifficulty;
        });
      }
    });
  }

  takescrshot() async {
    RenderRepaintBoundary boundary =
        previewContainer.currentContext.findRenderObject();
    var image = await boundary.toImage();
    var byteData = await image.toByteData(format: ImageByteFormat.png);
    var pngBytes = byteData.buffer.asUint8List();
    img.Image background = img.decodeImage(pngBytes);

    final qrFile = File("assets/images/starcoin-qr.png");
    img.Image qr = img.decodeImage(qrFile.readAsBytesSync());

    img.drawImage(background, qr, dstX: 40, dstY: 450, dstH: 120, dstW: 120);
    // String fileName = DateTime.now().toIso8601String();
    // var path =
    //     '/Users/fanngyuan/Documents/workspace/starcoin_node_gui/$fileName.png';
    // //final file = File(path);
    // //await file.writeAsBytes(wmImage);
    // File(path)..writeAsBytesSync(ui.encodePng(Img));

    //String fileName = DateTime.now().toIso8601String();
    //var path =
    //    '/Users/fanngyuan/Documents/workspace/starcoin_node_gui/$fileName.png';
    //final file = File(path);
    //await file.writeAsBytes(pngBytes);

    // final _originalImage = File("assets/images/starcoin-share-template.png");
    // ui.Image Img = ui.decodeImage(_originalImage.readAsBytesSync());
    // ui.drawString(Img, ui.arial_48, 800, 400, 'Add Text 123',
    //     color: 0xff00ffff);
    int fileName = DateTime.now().microsecondsSinceEpoch;

    var dir;
    if (Platform.isMacOS){
        final current = await DirectoryService.getCurrentDirectory();
        dir = Directory.fromUri(Uri.parse(current)).parent.path;
    
    }
    if (Platform.isWindows){
        Directory current = Directory.current;
        dir = current.path;
    }

    var path =join(dir,'$fileName.png');
    // //final file = File(path);
    // //await file.writeAsBytes(wmImage);
    File(path)..writeAsBytesSync(img.encodePng(background));
  }
}
