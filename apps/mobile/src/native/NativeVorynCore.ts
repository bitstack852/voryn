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
}

export default TurboModuleRegistry.get<Spec>('VorynCore');
