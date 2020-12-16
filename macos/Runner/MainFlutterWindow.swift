import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow {

  private var methodChannel: FlutterMethodChannel?
  
  override func awakeFromNib() {
    let flutterViewController = FlutterViewController.init()
    let windowFrame = self.frame
    self.contentViewController = flutterViewController
    self.setFrame(windowFrame, display: true)

    methodChannel = FlutterMethodChannel(name: "wallet.starcoin.org/channel", binaryMessenger: flutterViewController.engine.binaryMessenger)
    
    methodChannel?.setMethodCallHandler({ (call: FlutterMethodCall, result: FlutterResult) in
      guard call.method == "getCurrentDirectory" else {
        result(FlutterMethodNotImplemented)
        return
      }
      let path = Bundle.main.bundlePath;
      result(path);
    })    

    RegisterGeneratedPlugins(registry: flutterViewController)

    super.awakeFromNib()
  }
}