import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:starcoin_node/pages/node_page.dart';
import 'package:starcoin_node/pages/routes/routes.dart';

class IntroPage extends StatelessWidget {
  static const String routeName = Routes.main + "/intro";

  final nameController = TextEditingController();

  @override
  Widget build(BuildContext context) {
    final blue = Color.fromARGB(255, 0, 255, 255);
    return Scaffold(
        body: Container(
            alignment: Alignment(0, 0),
            decoration: BoxDecoration(
                image: DecorationImage(
                    image: AssetImage("assets/images/starcoin-bg.png"),
                    fit: BoxFit.cover)),
            child: Center(
              child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.center,
                  children: <Widget>[
                    Container(
                        width: 200.0,
                        child: TextField(
                          style: TextStyle(color: Colors.white),
                          controller: nameController,
                          decoration: InputDecoration(
                              border: OutlineInputBorder(
                                borderSide: const BorderSide(
                                    color: Colors.blue, width: 1),
                              ),
                              focusedBorder: OutlineInputBorder(
                                borderRadius:
                                    BorderRadius.all(Radius.circular(4)),
                                borderSide: BorderSide(width: 1, color: blue),
                              ),
                              hintText: '创建昵称'),
                        )),
                    SizedBox(
                      height: 20,
                    ),
                    RaisedButton(
                      onPressed: () async {
                        final sharedPrefs =
                            await SharedPreferences.getInstance();
                        final name = nameController.text;
                        await sharedPrefs.setString("user_name", name);
                        Navigator.of(context)
                            .push(new MaterialPageRoute(builder: (_) {
                          return new NodePage(name);
                        }));
                      },
                      child: const Text('确认', style: TextStyle(fontSize: 30)),
                      shape: RoundedRectangleBorder(
                        borderRadius: new BorderRadius.circular(20.0),
                        side: BorderSide(color: blue),
                      ),
                    )
                  ]),
            )));
  }
}
