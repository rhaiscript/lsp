export interface Config {
  source?: SourceConfig;
}

export interface SourceConfig {
  include?: string[];
  exclude?: string[];
}
