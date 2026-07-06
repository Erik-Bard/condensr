export interface ShortenResponse {
  code: string;
  short_url: string;
  long_url: string;
}

export interface LinkItem {
  id: number;
  long_url: string;
  created_at: string;
  code: string;
  short_url: string;
}
