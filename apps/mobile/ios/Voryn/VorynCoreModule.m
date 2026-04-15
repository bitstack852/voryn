#import <React/RCTBridgeModule.h>
#import "voryn_core.h"

@interface VorynCoreModule : NSObject <RCTBridgeModule>
@end

@implementation VorynCoreModule

RCT_EXPORT_MODULE(VorynCore);

RCT_EXPORT_METHOD(hello:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject) {
  const char* result = voryn_hello();
  NSString* str = [NSString stringWithUTF8String:result];
  voryn_free_string(result);
  resolve(str);
}

RCT_EXPORT_METHOD(generateIdentity:(RCTPromiseResolveBlock)resolve reject:(RCTPromiseRejectBlock)reject) {
  const char* result = voryn_generate_identity();
  NSString* json = [NSString stringWithUTF8String:result];
  voryn_free_string(result);
  resolve(json);
}

@end
