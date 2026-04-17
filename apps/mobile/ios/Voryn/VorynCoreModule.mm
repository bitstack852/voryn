#import <React/RCTBridgeModule.h>
#import "voryn_core.h"
#import "VorynCoreSpec/VorynCoreSpec.h"

@interface VorynCoreModule : NSObject <NativeVorynCoreSpec>
@end

@implementation VorynCoreModule

RCT_EXPORT_MODULE(VorynCore);

// ── Crypto ──────────────────────────────────────────────────────────

RCT_EXPORT_METHOD(hello:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject) {
  const char* result = voryn_hello();
  resolve([NSString stringWithUTF8String:result]);
  voryn_free_string(result);
}

RCT_EXPORT_METHOD(generateIdentity:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject) {
  const char* result = voryn_generate_identity();
  resolve([NSString stringWithUTF8String:result]);
  voryn_free_string(result);
}

// ── Network ─────────────────────────────────────────────────────────

RCT_EXPORT_METHOD(startNode:(NSString *)configJson
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject) {
  dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
    const char* result = voryn_start_node([configJson UTF8String]);
    NSString* json = [NSString stringWithUTF8String:result];
    voryn_free_string(result);
    resolve(json);
  });
}

RCT_EXPORT_METHOD(stopNode:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject) {
  dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
    const char* result = voryn_stop_node();
    NSString* json = [NSString stringWithUTF8String:result];
    voryn_free_string(result);
    resolve(json);
  });
}

RCT_EXPORT_METHOD(sendMessage:(NSString *)peerId
                  dataHex:(NSString *)dataHex
                  resolve:(RCTPromiseResolveBlock)resolve
                  reject:(RCTPromiseRejectBlock)reject) {
  dispatch_async(dispatch_get_global_queue(DISPATCH_QUEUE_PRIORITY_DEFAULT, 0), ^{
    NSUInteger hexLen = [dataHex length];
    NSMutableData* data = [NSMutableData dataWithCapacity:hexLen / 2];
    for (NSUInteger i = 0; i < hexLen; i += 2) {
      NSString* byteStr = [dataHex substringWithRange:NSMakeRange(i, 2)];
      unsigned int byte = 0;
      [[NSScanner scannerWithString:byteStr] scanHexInt:&byte];
      uint8_t b = (uint8_t)byte;
      [data appendBytes:&b length:1];
    }
    const char* result = voryn_send_message(
      [peerId UTF8String],
      (const uint8_t*)[data bytes],
      [data length]
    );
    NSString* json = [NSString stringWithUTF8String:result];
    voryn_free_string(result);
    resolve(json);
  });
}

RCT_EXPORT_METHOD(pollEvent:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject) {
  const char* result = voryn_poll_event();
  if (result == NULL) {
    resolve([NSNull null]);
  } else {
    NSString* json = [NSString stringWithUTF8String:result];
    voryn_free_string(result);
    resolve(json);
  }
}

RCT_EXPORT_METHOD(nodeStatus:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject) {
  const char* result = voryn_node_status();
  NSString* json = [NSString stringWithUTF8String:result];
  voryn_free_string(result);
  resolve(json);
}

- (std::shared_ptr<facebook::react::TurboModule>)getTurboModule:(const facebook::react::ObjCTurboModule::InitParams &)params {
  return std::make_shared<facebook::react::NativeVorynCoreSpecJSI>(params);
}

@end
