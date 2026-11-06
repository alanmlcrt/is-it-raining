use chrono::{DateTime, Duration, Utc};
use serde::Serialize;

pub const STEP_MINUTES: i64 = 5;
const SOURCE: &str = "piaf";
/// Pas de la grille PIAF, en degrés.
const RESOLUTION: f64 = 0.01;

/// Un run PIAF complet. `steps[i]` couvre `[run_at + 5i, run_at + 5(i + 1)]` minutes, en mm/h.
pub struct Grid {
    pub run_at: DateTime<Utc>,
    /// Latitude de la première ligne, la plus au nord.
    pub lat0: f64,
    /// Longitude de la première colonne, la plus à l'ouest.
    pub lon0: f64,
    pub ni: usize,
    pub nj: usize,
    pub steps: Vec<Vec<f32>>,
}

impl Grid {
    /// Série au point de grille le plus proche. `None` hors de la grille.
    fn sample(&self, lat: f64, lon: f64) -> Option<Vec<f32>> {
        let row = ((self.lat0 - lat) / RESOLUTION).round();
        let col = ((lon - self.lon0) / RESOLUTION).round();
        if !(0.0..self.nj as f64).contains(&row) || !(0.0..self.ni as f64).contains(&col) {
            return None;
        }
        let i = row as usize * self.ni + col as usize;
        Some(self.steps.iter().map(|step| step[i]).collect())
    }
}

#[derive(Clone)]
pub struct Params {
    pub rain_threshold_mm_h: f32,
    pub min_wet_steps: usize,
    pub max_data_age: Duration,
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Dry,
    RainSoon,
    Raining,
    Unknown,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Step {
    /// Fin du pas : la valeur est un cumul sur les 5 minutes qui précèdent.
    pub t: DateTime<Utc>,
    pub mm_h: f32,
}

#[derive(Debug, Serialize)]
pub struct Rain {
    pub state: State,
    pub source: Option<&'static str>,
    pub step_minutes: Option<i64>,
    pub run_at: Option<DateTime<Utc>>,
    pub rain_starts_at: Option<DateTime<Utc>>,
    pub rain_ends_at: Option<DateTime<Utc>>,
    pub horizon_ends_at: Option<DateTime<Utc>>,
    pub series: Vec<Step>,
}

impl Rain {
    /// Pas de série : un client ne doit rien pouvoir afficher comme du temps sec.
    fn unknown(run_at: Option<DateTime<Utc>>) -> Self {
        Rain {
            state: State::Unknown,
            source: run_at.map(|_| SOURCE),
            step_minutes: None,
            run_at,
            rain_starts_at: None,
            rain_ends_at: None,
            horizon_ends_at: None,
            series: Vec::new(),
        }
    }
}

/// Plages `[début, fin[` d'au moins `min_steps` pas pluvieux consécutifs.
///
/// `ponytail:` un seul pas sec coupe un épisode. Lisser avec `DRY_RESET_MIN` si `rain_ends_at`
/// paraît trop tôt sur le terrain (calibration N4).
fn episodes(wet: &[bool], min_steps: usize) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut start = None;
    for i in 0..=wet.len() {
        match (wet.get(i).copied().unwrap_or(false), start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                if i - s >= min_steps {
                    found.push((s, i));
                }
                start = None;
            }
            _ => {}
        }
    }
    found
}

/// La réponse pour un point. `unknown` dès qu'on ne peut pas affirmer quelque chose.
pub fn rain_at(grid: Option<&Grid>, lat: f64, lon: f64, now: DateTime<Utc>, p: &Params) -> Rain {
    let Some(grid) = grid else {
        return Rain::unknown(None);
    };
    // Hors grille : rien de PIAF à citer. Le repli Open-Meteo viendra ici (A3).
    let Some(mm_h) = grid.sample(lat, lon) else {
        return Rain::unknown(None);
    };
    if now - grid.run_at > p.max_data_age || mm_h.is_empty() || mm_h.iter().any(|v| !v.is_finite())
    {
        return Rain::unknown(Some(grid.run_at));
    }

    let at = |i: usize| grid.run_at + Duration::minutes(STEP_MINUTES * i as i64);
    let wet: Vec<bool> = mm_h.iter().map(|v| *v >= p.rain_threshold_mm_h).collect();
    // Épisodes cherchés sur toute la série, passé compris : les derniers pas d'une averse en
    // cours comptent, même s'il en reste moins que `min_wet_steps` devant nous.
    let next = episodes(&wet, p.min_wet_steps)
        .into_iter()
        .find(|(_, end)| at(*end) > now);

    let state = match next {
        None => State::Dry,
        Some((start, _)) if at(start) <= now => State::Raining,
        Some(_) => State::RainSoon,
    };
    Rain {
        state,
        source: Some(SOURCE),
        step_minutes: Some(STEP_MINUTES),
        run_at: Some(grid.run_at),
        rain_starts_at: next.map(|(start, _)| at(start)),
        // Pas de fin annoncée si la pluie dure jusqu'au bout de l'horizon.
        rain_ends_at: next.and_then(|(_, end)| (end < mm_h.len()).then(|| at(end))),
        horizon_ends_at: Some(at(mm_h.len())),
        series: mm_h
            .iter()
            .enumerate()
            .map(|(i, v)| Step {
                t: at(i + 1),
                // Deux décimales : PIAF ne descend pas sous 0,12 mm/h.
                mm_h: (v * 100.0).round() / 100.0,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 1.2;
    const D: f32 = 0.0;

    fn params() -> Params {
        Params {
            rain_threshold_mm_h: 0.1,
            min_wet_steps: 2,
            max_data_age: Duration::minutes(20),
        }
    }

    fn run_at() -> DateTime<Utc> {
        "2026-11-05T14:30:00Z".parse().unwrap()
    }

    /// Grille d'un seul point, à (48.86, 2.35), qui suit `series`. Le reste de l'horizon est sec.
    fn grid(series: &[f32]) -> Grid {
        let mut steps: Vec<Vec<f32>> = series.iter().map(|v| vec![*v]).collect();
        steps.resize(39, vec![D]);
        Grid {
            run_at: run_at(),
            lat0: 48.86,
            lon0: 2.35,
            ni: 1,
            nj: 1,
            steps,
        }
    }

    fn at_min(series: &[f32], minutes: i64) -> Rain {
        let now = run_at() + Duration::minutes(minutes);
        rain_at(Some(&grid(series)), 48.86, 2.35, now, &params())
    }

    fn t(minutes: i64) -> Option<DateTime<Utc>> {
        Some(run_at() + Duration::minutes(minutes))
    }

    #[test]
    fn sec_sur_tout_l_horizon() {
        let r = at_min(&[], 12);
        assert_eq!(r.state, State::Dry);
        assert_eq!(r.rain_starts_at, None);
        assert_eq!(r.horizon_ends_at, t(195));
        assert_eq!(r.series.len(), 39);
        assert_eq!(r.series[0].t, t(5).unwrap());
    }

    #[test]
    fn averse_d_un_pas_ignoree() {
        assert_eq!(at_min(&[D, D, D, D, W, D], 12).state, State::Dry);
    }

    #[test]
    fn debut_et_fin_de_pluie() {
        // Pas d'indice 5 à 7 : pluie de +25 à +40 min.
        let r = at_min(&[D, D, D, D, D, W, W, W], 12);
        assert_eq!(r.state, State::RainSoon);
        assert_eq!(r.rain_starts_at, t(25));
        assert_eq!(r.rain_ends_at, t(40));
    }

    #[test]
    fn pluie_en_cours() {
        let r = at_min(&[W, W, W, W, W, D], 12);
        assert_eq!(r.state, State::Raining);
        assert_eq!(r.rain_ends_at, t(25));
    }

    #[test]
    fn pluie_sans_fin_dans_l_horizon() {
        let r = at_min(&[W; 39], 12);
        assert_eq!(r.state, State::Raining);
        assert_eq!(r.rain_ends_at, None);
    }

    #[test]
    fn fin_d_averse_encore_en_cours() {
        // Il ne reste qu'un pas pluvieux devant nous : il pleut toujours.
        let r = at_min(&[W, W, W, D], 12);
        assert_eq!(r.state, State::Raining);
        assert_eq!(r.rain_ends_at, t(15));
    }

    #[test]
    fn averse_passee_ignoree() {
        let r = at_min(&[W, W, D, D, D, D, W, W], 12);
        assert_eq!(r.state, State::RainSoon);
        assert_eq!(r.rain_starts_at, t(30));
        assert_eq!(at_min(&[W, W], 12).state, State::Dry);
    }

    #[test]
    fn seuil_de_pluie() {
        assert_eq!(at_min(&[0.09; 39], 12).state, State::Dry);
        assert_eq!(at_min(&[0.12; 39], 12).state, State::Raining);
    }

    #[test]
    fn run_perime_jamais_sec() {
        assert_eq!(at_min(&[], 20).state, State::Dry);
        let r = at_min(&[], 21);
        assert_eq!(r.state, State::Unknown);
        assert_eq!(r.run_at, t(0));
        assert!(r.series.is_empty());
    }

    #[test]
    fn sans_grille_inconnu() {
        let r = rain_at(None, 48.86, 2.35, run_at(), &params());
        assert_eq!(r.state, State::Unknown);
        assert_eq!(r.source, None);
    }

    #[test]
    fn hors_grille_inconnu() {
        let r = rain_at(Some(&grid(&[])), 40.0, 2.35, run_at(), &params());
        assert_eq!(r.state, State::Unknown);
        assert_eq!(r.source, None);
        // Run périmé en plus : toujours rien à attribuer à PIAF.
        let late = run_at() + Duration::minutes(30);
        let r = rain_at(Some(&grid(&[])), 40.0, 2.35, late, &params());
        assert_eq!(r.source, None);
    }

    #[test]
    fn valeur_manquante_jamais_seche() {
        assert_eq!(at_min(&[D, f32::NAN], 12).state, State::Unknown);
    }

    #[test]
    fn echantillonnage_ligne_colonne() {
        // 3 colonnes, 2 lignes : nord en haut, point le plus proche.
        let g = Grid {
            run_at: run_at(),
            lat0: 51.5,
            lon0: -6.0,
            ni: 3,
            nj: 2,
            steps: vec![vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0]],
        };
        assert_eq!(g.sample(51.5, -6.0), Some(vec![0.0]));
        assert_eq!(g.sample(51.5, -5.98), Some(vec![2.0]));
        assert_eq!(g.sample(51.49, -5.99), Some(vec![4.0]));
        assert_eq!(g.sample(51.4904, -5.9904), Some(vec![4.0]));
        assert_eq!(g.sample(51.47, -6.0), None);
        assert_eq!(g.sample(51.5, -5.97), None);
    }
}
