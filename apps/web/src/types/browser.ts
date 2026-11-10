/** Optional browser capability; unsupported devices retain the manual approval-link flow. */
export interface BarcodeDetectorConstructor {
  new (options: { formats: string[] }): {
    detect(source: HTMLVideoElement): Promise<{ rawValue: string }[]>;
  };
}
