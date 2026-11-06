// Le lieu actif (D7) et la recherche de ville, sans React pour rester testable.

export type Place = { name: string; lat: number; lon: number };
export type City = Place & { id: number; region: string };

export const MIN_QUERY = 2;
const SEARCH_URL = 'https://geocoding-api.open-meteo.com/v1/search';

/** Vie privée : le client n'envoie jamais plus de 2 décimales, environ 1 km. */
export function round2(value: number): number {
  return Math.round(value * 100) / 100;
}

export function searchUrl(query: string): string {
  return `${SEARCH_URL}?name=${encodeURIComponent(query.trim())}&language=fr&count=5`;
}

type Result = {
  id: number;
  name: string;
  latitude: number;
  longitude: number;
  admin1?: string;
  country?: string;
};

/** `results` est absent quand rien ne correspond. */
export function parseCities(body: { results?: Result[] }): City[] {
  return (body.results ?? []).map((r) => ({
    id: r.id,
    name: r.name,
    lat: round2(r.latitude),
    lon: round2(r.longitude),
    region: [r.admin1, r.country].filter(Boolean).join(', '),
  }));
}
