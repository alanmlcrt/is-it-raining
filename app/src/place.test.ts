import assert from 'node:assert/strict';
import { test } from 'node:test';

import { parseCities, round2, searchUrl } from './place.ts';

test('deux décimales au plus', () => {
  assert.equal(round2(48.85661), 48.86);
  assert.equal(round2(-1.2345), -1.23);
});

test('la recherche échappe la saisie', () => {
  assert.match(searchUrl(' saint malo '), /name=saint%20malo&language=fr&count=5$/);
});

test('une ville garde sa région et perd la précision', () => {
  const [city] = parseCities({
    results: [
      {
        id: 1,
        name: 'Saint-Malo',
        latitude: 48.64811,
        longitude: -2.00889,
        admin1: 'Bretagne',
        country: 'France',
      },
    ],
  });
  assert.deepEqual(city, {
    id: 1,
    name: 'Saint-Malo',
    lat: 48.65,
    lon: -2.01,
    region: 'Bretagne, France',
  });
});

test('aucun résultat ou région incomplète', () => {
  assert.deepEqual(parseCities({}), []);
  const [city] = parseCities({ results: [{ id: 2, name: 'X', latitude: 0, longitude: 0 }] });
  assert.equal(city.region, '');
});
