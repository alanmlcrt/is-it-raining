import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  answer,
  barHeight,
  duration,
  minutesUntil,
  type Rain,
  sourceLine,
  spokenAnswer,
  upcoming,
} from './rain.ts';

const NOW = new Date('2026-11-05T14:45:00Z');

function rain(fields: Partial<Rain>): Rain {
  return {
    state: 'dry',
    source: 'piaf',
    step_minutes: 5,
    run_at: '2026-11-05T14:30:00Z',
    rain_starts_at: null,
    rain_ends_at: null,
    horizon_ends_at: '2026-11-05T17:45:00Z',
    series: [],
    ...fields,
  };
}

test('arrondi vers le haut au pas de la source', () => {
  assert.equal(minutesUntil('2026-11-05T15:05:00Z', NOW, 5), 20);
  assert.equal(minutesUntil('2026-11-05T15:05:00Z', new Date('2026-11-05T14:44:30Z'), 5), 25);
  assert.equal(minutesUntil('2026-11-05T15:05:00Z', new Date('2026-11-05T15:04:59Z'), 5), 5);
  assert.equal(minutesUntil('2026-11-05T15:15:00Z', new Date('2026-11-05T14:44:00Z'), 15), 45);
});

test('durées', () => {
  assert.equal(duration(20), '20 min');
  assert.equal(duration(100), '1 h 40');
  assert.equal(duration(65), '1 h 05');
  assert.equal(duration(120), '2 h');
});

test('rien sans réponse du serveur, jamais de temps sec', () => {
  assert.deepEqual(answer(null, NOW), { lead: 'Prévision indisponible' });
  assert.deepEqual(answer(rain({ state: 'unknown', horizon_ends_at: null }), NOW), {
    lead: 'Prévision indisponible',
  });
});

test('pluie proche, puis réponse dépassée pendant qu’elle reste à l’écran', () => {
  const soon = rain({
    state: 'rain_soon',
    rain_starts_at: '2026-11-05T15:05:00Z',
    rain_ends_at: '2026-11-05T15:50:00Z',
  });
  const a = answer(soon, NOW);
  assert.equal(a.lead, 'Pluie dans');
  assert.equal(a.big, '20 min');
  assert.match(a.detail ?? '', /^Jusque vers /);
  assert.equal(answer(soon, new Date('2026-11-05T15:06:00Z')).lead, 'Il pleut. Fin vers');
});

test('pluie sans fin dans l’horizon', () => {
  const a = answer(rain({ state: 'raining', rain_starts_at: '2026-11-05T14:30:00Z' }), NOW);
  assert.deepEqual(a, { lead: 'Il pleut pour au moins', big: '3 h' });
});

test('repli à 15 min : arrondi, approché et attribué à Open-Meteo', () => {
  const soon = rain({
    state: 'rain_soon',
    source: 'open_meteo',
    step_minutes: 15,
    rain_starts_at: '2026-11-05T15:05:00Z',
  });
  const a = answer(soon, NOW);
  assert.equal(a.big, '~30 min');
  assert.equal(spokenAnswer(a), 'Pluie dans environ 30 minutes');
  assert.match(sourceLine(soon, NOW), /^Open-Meteo, /);
});

test('la source reste affichée, même sans donnée', () => {
  assert.equal(sourceLine(rain({}), NOW), 'Météo-France, il y a 15 min');
  assert.equal(sourceLine(null, NOW), 'Météo-France, aucune donnée récente');
});

test('seuls les pas à venir sont dessinés', () => {
  const series = [
    { t: '2026-11-05T14:45:00Z', mm_h: 1 },
    { t: '2026-11-05T14:50:00Z', mm_h: 2 },
  ];
  assert.deepEqual(upcoming(rain({ series }), NOW), [series[1]]);
  assert.deepEqual(upcoming(rain({ state: 'unknown', series }), NOW), []);
});

test('barres : rien à 0, une pluie faible reste visible, plafond à 1', () => {
  assert.equal(barHeight(0), 0);
  assert.ok(barHeight(0.12) >= 0.08);
  assert.equal(barHeight(50), 1);
});

test('phrase du lecteur d’écran', () => {
  const spoken = spokenAnswer({ lead: 'Pluie dans', big: '20 min', detail: 'Jusque vers 15 h 50' });
  assert.equal(spoken, 'Pluie dans 20 minutes Jusque vers 15 heures 50');
});
