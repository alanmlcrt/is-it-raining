// Mise en forme d'une réponse de /v1/rain. Aucune logique de pluie ici : le serveur décide de
// l'état, ce fichier ne fait que `rain_starts_at - now` et du texte.

export type RainState = 'dry' | 'rain_soon' | 'raining' | 'unknown';

export type Step = { t: string; mm_h: number };

export type Rain = {
  state: RainState;
  source: string | null;
  step_minutes: number | null;
  run_at: string | null;
  rain_starts_at: string | null;
  rain_ends_at: string | null;
  horizon_ends_at: string | null;
  series: Step[];
};

export type Answer = { lead: string; big?: string; detail?: string };

const MINUTE = 60_000;
const SOURCE_NAMES: Record<string, string> = { piaf: 'Météo-France', open_meteo: 'Open-Meteo' };
/** Pluie forte en haut de l'échelle des barres, en mm/h. */
const BAR_SCALE_MM_H = 10;

export function clock(iso: string): string {
  const date = new Date(iso);
  const minutes = date.getMinutes();
  const hours = `${date.getHours()} h`;
  return minutes === 0 ? hours : `${hours} ${String(minutes).padStart(2, '0')}`;
}

export function duration(minutes: number): string {
  if (minutes < 60) return `${minutes} min`;
  const rest = minutes % 60;
  const hours = `${Math.floor(minutes / 60)} h`;
  return rest === 0 ? hours : `${hours} ${String(rest).padStart(2, '0')}`;
}

/** Minutes avant `iso`, arrondies vers le haut au pas de la source. */
export function minutesUntil(iso: string, now: Date, stepMinutes: number): number {
  const minutes = (new Date(iso).getTime() - now.getTime()) / MINUTE;
  return Math.ceil(minutes / stepMinutes) * stepMinutes;
}

/** `rain` à `null` : pas de réponse du serveur, on n'affiche rien d'ancien. */
export function answer(rain: Rain | null, now: Date): Answer {
  if (rain === null || rain.state === 'unknown' || rain.horizon_ends_at === null) {
    return { lead: 'Prévision indisponible' };
  }
  if (rain.state === 'dry') {
    return { lead: 'Pas de pluie avant', big: clock(rain.horizon_ends_at) };
  }
  const start = rain.rain_starts_at;
  const step = rain.step_minutes ?? 5;
  if (rain.state === 'rain_soon' && start !== null && new Date(start) > now) {
    return {
      lead: 'Pluie dans',
      // Pas de fausse précision : une source à 15 min s'affiche approchée.
      big: `${step > 5 ? '~' : ''}${duration(minutesUntil(start, now, step))}`,
      detail: rain.rain_ends_at === null ? undefined : `Jusque vers ${clock(rain.rain_ends_at)}`,
    };
  }
  return rain.rain_ends_at === null
    ? { lead: 'Il pleut pour au moins', big: '3 h' }
    : { lead: 'Il pleut. Fin vers', big: clock(rain.rain_ends_at) };
}

/** Attribution exigée par la licence : toujours affichée. */
export function sourceLine(rain: Rain | null, now: Date): string {
  const name = SOURCE_NAMES[rain?.source ?? ''] ?? 'Météo-France';
  if (!rain?.run_at) return `${name}, aucune donnée récente`;
  const age = Math.max(0, Math.floor((now.getTime() - new Date(rain.run_at).getTime()) / MINUTE));
  return `${name}, il y a ${duration(age)}`;
}

/** Les pas encore à venir. Le jaune veut dire pluie : rien à dessiner si le serveur dit sec. */
export function upcoming(rain: Rain | null, now: Date): Step[] {
  if (rain === null || rain.state === 'unknown') return [];
  return rain.series.filter((step) => new Date(step.t) > now);
}

/** Hauteur d'une barre, de 0 à 1. Racine carrée : une pluie faible doit rester visible. */
export function barHeight(mmH: number): number {
  if (mmH <= 0) return 0;
  return Math.max(0.08, Math.min(1, Math.sqrt(mmH / BAR_SCALE_MM_H)));
}

/** La bande pour un lecteur d'écran : une phrase, pas barre par barre. */
export function bandSentence(rain: Rain | null): string {
  if (rain?.rain_starts_at) {
    const end = rain.rain_ends_at ? ` à ${clock(rain.rain_ends_at)}` : '';
    return speak(`Pluie prévue à partir de ${clock(rain.rain_starts_at)}${end}`);
  }
  return rain?.horizon_ends_at
    ? speak(`Aucune pluie prévue avant ${clock(rain.horizon_ends_at)}`)
    : 'Pas de prévision';
}

/** Une réponse en phrase pour un lecteur d'écran. */
export function spokenAnswer({ lead, big, detail }: Answer): string {
  return speak([lead, big, detail].filter(Boolean).join(' '));
}

function speak(text: string): string {
  return text
    .replace(/~/g, 'environ ')
    .replace(/(\d+) h (\d+)/g, '$1 heures $2')
    .replace(/(\d+) h\b/g, '$1 heures')
    .replace(/(\d+) min\b/g, '$1 minutes');
}
