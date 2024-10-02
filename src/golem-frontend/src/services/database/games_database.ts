export interface GamesDatabaseRow {
  id: number;
  system_id: number;
  catalog_id: number;
  unique_id: string;
  name: string;
  description: string;

  manufacturer: string | null;
}

export class GamesDatabase {}
