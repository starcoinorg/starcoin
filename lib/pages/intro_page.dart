import 'package:flutter/material.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:starcoin_node/pages/node_page.dart';
import 'package:starcoin_node/pages/routes/routes.dart';

class IntroPage extends StatelessWidget {
  static const String routeName = Routes.main + "/intro";

  final nameController = TextEditingController();

  @override
  Widget build(BuildContext context) {
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
                              border: OutlineInputBorder(),
                              hintText: 'Enter Name'),
                        )),
                    SizedBox(
                      height: 10,
                    ),
                    RaisedButton(
                      onPressed: () async {
                        final sharedPrefs =
                            await SharedPreferences.getInstance();
                        final name = nameController.text;
                        await sharedPrefs.setString("user_name", name);
                        Navigator.pushNamed(context, NodePage.routeName);
                        Navigator.of(context)
                            .push(new MaterialPageRoute(builder: (_) {
                          return new NodePage(name);
                        }));
                      },
                      child:
                          const Text('Confirm', style: TextStyle(fontSize: 30)),
                    )
                  ]),
            )));
  }
}
