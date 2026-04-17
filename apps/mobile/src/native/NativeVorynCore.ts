import type {TurboModule} from 'react-native';
import {TurboModuleRegistry} from 'react-native';

export interface Spec extends TurboModule {
  hello(): Promise<string>;
  generateIdentity(): Promise<string>;
  startNode(configJson: string): Promise<string>;
  stopNode(): Promise<string>;
  pollEvent(): Promise<string | null>;
  nodeStatus(): Promise<string>;
  sendMessage(peerId: string, dataHex: string): Promise<string>;
  encryptMessage(plaintext: string, ourSecretKeyHex: string, ourPublicKeyHex: string, theirPublicKeyHex: string): Promise<string>;
  decryptMessage(envelopeHex: string, ourSecretKeyHex: string): Promise<string>;
  peerIdFromPublicKey(publicKeyHex: string): Promise<string>;
}

export default TurboModuleRegistry.get<Spec>('VorynCore');
