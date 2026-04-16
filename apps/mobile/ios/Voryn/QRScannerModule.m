#import <React/RCTBridgeModule.h>
#import <AVFoundation/AVFoundation.h>
#import <UIKit/UIKit.h>

@interface QRScannerModule : NSObject <RCTBridgeModule, AVCaptureMetadataOutputObjectsDelegate>
@property (nonatomic, strong) AVCaptureSession *captureSession;
@property (nonatomic, copy) RCTPromiseResolveBlock resolveBlock;
@property (nonatomic, copy) RCTPromiseRejectBlock rejectBlock;
@property (nonatomic, strong) UIViewController *scannerVC;
@end

@implementation QRScannerModule

RCT_EXPORT_MODULE(QRScanner);

RCT_EXPORT_METHOD(scan:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject) {
  self.resolveBlock = resolve;
  self.rejectBlock = reject;

  dispatch_async(dispatch_get_main_queue(), ^{
    AVAuthorizationStatus status = [AVCaptureDevice authorizationStatusForMediaType:AVMediaTypeVideo];

    if (status == AVAuthorizationStatusDenied || status == AVAuthorizationStatusRestricted) {
      reject(@"CAMERA_DENIED", @"Camera access denied", nil);
      return;
    }

    if (status == AVAuthorizationStatusNotDetermined) {
      [AVCaptureDevice requestAccessForMediaType:AVMediaTypeVideo completionHandler:^(BOOL granted) {
        if (granted) {
          dispatch_async(dispatch_get_main_queue(), ^{
            [self presentScanner];
          });
        } else {
          reject(@"CAMERA_DENIED", @"Camera access denied", nil);
        }
      }];
      return;
    }

    [self presentScanner];
  });
}

- (void)presentScanner {
  self.captureSession = [[AVCaptureSession alloc] init];

  AVCaptureDevice *device = [AVCaptureDevice defaultDeviceWithMediaType:AVMediaTypeVideo];
  if (!device) {
    if (self.rejectBlock) self.rejectBlock(@"NO_CAMERA", @"No camera available", nil);
    return;
  }

  NSError *error = nil;
  AVCaptureDeviceInput *input = [AVCaptureDeviceInput deviceInputWithDevice:device error:&error];
  if (error || !input) {
    if (self.rejectBlock) self.rejectBlock(@"CAMERA_ERROR", error.localizedDescription, error);
    return;
  }

  [self.captureSession addInput:input];

  AVCaptureMetadataOutput *output = [[AVCaptureMetadataOutput alloc] init];
  [self.captureSession addOutput:output];
  [output setMetadataObjectsDelegate:self queue:dispatch_get_main_queue()];
  output.metadataObjectTypes = @[AVMetadataObjectTypeQRCode];

  AVCaptureVideoPreviewLayer *previewLayer = [AVCaptureVideoPreviewLayer layerWithSession:self.captureSession];
  previewLayer.videoGravity = AVLayerVideoGravityResizeAspectFill;

  // Create scanner view controller
  UIViewController *vc = [[UIViewController alloc] init];
  vc.view.backgroundColor = [UIColor blackColor];
  previewLayer.frame = vc.view.bounds;
  [vc.view.layer addSublayer:previewLayer];

  // Add close button
  UIButton *closeBtn = [UIButton buttonWithType:UIButtonTypeSystem];
  [closeBtn setTitle:@"Cancel" forState:UIControlStateNormal];
  [closeBtn setTitleColor:[UIColor whiteColor] forState:UIControlStateNormal];
  closeBtn.titleLabel.font = [UIFont systemFontOfSize:18 weight:UIFontWeightSemibold];
  closeBtn.frame = CGRectMake(20, 60, 80, 40);
  [closeBtn addTarget:self action:@selector(closeScanner) forControlEvents:UIControlEventTouchUpInside];
  [vc.view addSubview:closeBtn];

  // Add scan area indicator
  UIView *scanFrame = [[UIView alloc] initWithFrame:CGRectMake(0, 0, 250, 250)];
  scanFrame.center = vc.view.center;
  scanFrame.layer.borderColor = [UIColor colorWithRed:0.29 green:0.62 blue:1.0 alpha:1.0].CGColor;
  scanFrame.layer.borderWidth = 2.0;
  scanFrame.layer.cornerRadius = 16;
  [vc.view addSubview:scanFrame];

  // Add instruction label
  UILabel *label = [[UILabel alloc] initWithFrame:CGRectMake(0, 0, 300, 40)];
  label.center = CGPointMake(vc.view.center.x, scanFrame.frame.origin.y - 40);
  label.text = @"Point at a Voryn QR code";
  label.textColor = [UIColor whiteColor];
  label.textAlignment = NSTextAlignmentCenter;
  label.font = [UIFont systemFontOfSize:16];
  [vc.view addSubview:label];

  self.scannerVC = vc;

  UIViewController *rootVC = [UIApplication sharedApplication].delegate.window.rootViewController;
  while (rootVC.presentedViewController) {
    rootVC = rootVC.presentedViewController;
  }

  vc.modalPresentationStyle = UIModalPresentationFullScreen;
  [rootVC presentViewController:vc animated:YES completion:^{
    [self.captureSession startRunning];
  }];
}

- (void)closeScanner {
  [self.captureSession stopRunning];
  [self.scannerVC dismissViewControllerAnimated:YES completion:nil];
  if (self.rejectBlock) {
    self.rejectBlock(@"CANCELLED", @"Scanner cancelled", nil);
    self.resolveBlock = nil;
    self.rejectBlock = nil;
  }
}

- (void)captureOutput:(AVCaptureOutput *)output didOutputMetadataObjects:(NSArray<__kindof AVMetadataObject *> *)metadataObjects fromConnection:(AVCaptureConnection *)connection {
  if (metadataObjects.count == 0) return;

  AVMetadataMachineReadableCodeObject *code = metadataObjects.firstObject;
  if ([code.type isEqualToString:AVMetadataObjectTypeQRCode] && code.stringValue) {
    [self.captureSession stopRunning];

    // Vibrate for feedback
    AudioServicesPlaySystemSound(kSystemSoundID_Vibrate);

    [self.scannerVC dismissViewControllerAnimated:YES completion:^{
      if (self.resolveBlock) {
        self.resolveBlock(code.stringValue);
        self.resolveBlock = nil;
        self.rejectBlock = nil;
      }
    }];
  }
}

@end
