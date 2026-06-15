# Logbook — Patina (passerelle de paiement Monero, Rust)

> Journal de bord du projet. But : garder une trace honnête de ce que je construis,
> de ce qui coince, de ce que j'apprends, et des retours de revue. On avance
> **par briques** : chaque brique doit compiler, tourner, et être notée ici avant
> de passer à la suivante.

---

## Comment je tiens ce journal

Une **entrée par session** de travail. Je remplis le modèle ci-dessous à chaud.
La règle : être franc sur ce qui a foiré ou ce que j'ai fait "à l'arrache" — c'est
là qu'est l'apprentissage, et c'est ce que la revue exploite. Embellir ne sert à rien.

Trois sections font le cœur de chaque entrée :

- **Ce qui a coincé** → mes vrais points de friction.
- **Décisions techniques** → ce que j'ai choisi *et pourquoi* (mon raisonnement, même bancal).
- **Revue du senior** → le retour critique + les corrections de cap.

---

## Roadmap

Logique d'amélioration : on part d'un truc minuscule qui marche, et on durcit
brique par brique jusqu'à un vrai service. Chaque phase est livrable seule.

- [x] **Phase 0 — Infra stagenet** : nœud (distant) + `monero-wallet-rpc` qui répond, fonds de test reçus.
- [x] **Phase 1 — v0 CLI** : générer une sous-adresse, détecter le paiement, attendre les confirmations.
- [ ] **Phase 2 — Persistance** : stocker les factures (SQLite), survivre à un redémarrage.
- [ ] **Phase 3 — API HTTP** (axum) : créer une facture, consulter son statut.
- [ ] **Phase 4 — Scanner de fond** (tokio) : machine à états, confirmations, idempotence.
- [ ] **Phase 5 — Webhooks** : notifier le marchand, signés + retries.
- [ ] **Phase 6 — Packaging** : Docker, config (env/fichier), logs/observabilité.
- [ ] **Phase 7 — Confidentialité** : nœud auto-hébergé, Tor, *threat model* documenté.
- [ ] **Phase 8 — Qualité** : tests, CI, README sérieux.
- [ ] **Niveau 2 (bonus)** : scan direct via la lib `monero-wallet` → internes Monero, rampe vers Cuprate.

---

## Modèle d'entrée (à copier-coller pour chaque session)

```
## Phase X — <titre> — <date>

**Objectif de la session :**

**Ce que j'ai fait :**

**Ce qui a coincé (honnête) :**

**Ce que j'ai appris :**

**Décisions techniques (et pourquoi) :**

**Revue du senior :**

**Prochaine étape :**
```

---

## Phase 0 — Mise en place de l'infra stagenet — *11/06/2026*

**Objectif de la session :**
Avoir `monero-wallet-rpc` qui tourne sur stagenet, connecté à un nœud, avec des
fonds de test, et confirmer que le RPC répond.

**Ce que j'ai fait :**

- Récupéré les binaires officiels Monero v0.18.5 (`monerod`, `monero-wallet-cli`, `monero-wallet-rpc`).
- Ajouté le dossier au `PATH`.
- Créé un wallet stagenet et lancé `monero-wallet-rpc` (port `38083`, `--disable-rpc-login`, en local).
- Récupéré des fonds via le faucet stagenet.
- Vérifié le RPC à la main (`create_address`, `get_balance`).

**Ce qui a coincé (honnête) :**

1. **PATH via copié-collé Stack Overflow.** J'ai collé dans `~/.bashrc` :
   `export PATH="/home/ant/project-monero/monero-x86_64-linux-gnu-v0.18.5.0:$PATH`
   sans trop comprendre, juste pour que ça marche.
nb : 13/06/2026, j'ai relu le poste stackoverflow, la syntaxe est claire
2. **Nœud stagenet fourni HS.** Le nœud `stagenet.community.rino.io:38081` ne répondait
   pas. J'ai cherché une liste de nœuds (`xmr.ditatompel.com/remote-nodes`) et basculé
   sur `node2.monerodevs.org:38089`, qui marche.

**Nœud utilisé actuellement :** `node2.monerodevs.org:38089` *(stagenet, port 38089)*

**Ce que j'ai appris :**

- La différence entre le **nœud** (la blockchain, port 38089 ici) et le **wallet-rpc**
  (mon wallet, port 38083 en local). Mon code parlera au wallet-rpc, pas au nœud.
- Les nœuds publics tombent — on ne peut pas en dépendre sérieusement.
- Nb : 13/06/2026 Je pense que ça ne change rien puisque mon wallet rpc contacte le noeud, par contre j'imagine qu'on peut donner une liste de noeud public au wallet rpc pour être plus resilient

**Décisions techniques (et pourquoi) :**

- Nœud **distant** pour le v0, pour éviter de synchroniser plusieurs Go.
  Conscient que c'est temporaire (cf. Phase 7).

**Revue du senior :**

- ✅ **Le bon réflexe de la session : le swap de nœud.** Diagnostiquer une dépendance
  morte dès la première heure et trouver soi-même une alternative qui marche, c'est
  *exactement* le métier. Et tu viens de vivre, en vrai, pourquoi "héberger son propre
  nœud" est sur la roadmap (Phase 7) : fiabilité **et** confidentialité. Garde ça en tête.
- ⚠️ **Le PATH versionné va te claquer dans les doigts.** Ton chemin contient le numéro
  de version (`...-v0.18.5.0`). À la prochaine mise à jour de Monero, ce dossier
  n'existera plus → `monerod` disparaît du PATH sans message d'erreur clair, et tu
  perds 30 min à comprendre pourquoi. **Mieux :** un lien symbolique non versionné, p.ex.
  `ln -s /home/ant/project-monero/monero-x86_64-linux-gnu-v0.18.5.0 ~/monero`, puis tu
  mets `~/monero` dans le PATH. Mise à jour = tu repointes le lien, le PATH ne bouge jamais.
  - Done + remboursement dette path
- ⚠️ **Vérifie le guillemet fermant.** La ligne que tu as collée n'a pas son `"` de fin
  (`...:$PATH` au lieu de `...:$PATH"`). Une guillemet ouverte non fermée casse le parsing
  du `.bashrc`. Si ton terminal s'ouvre normalement, c'est probablement juste une coquille
  - c'étaiyt bien juste une coquille de copier/collé
  recopiée ; sinon, c'est ton bug. À checker.
- 🧠 **Habitude à prendre, pas une faute :** comprendre une ligne *avant* de la coller,
  surtout quand elle touche au shell, au PATH, ou qu'elle commence par `sudo`. Ici c'était
  bénin. Un jour ça ne le sera pas. Le copié-collé qui marche est une dette, pas une dette nulle.

**Prochaine étape :** Phase 1 — écrire le v0 qui crée une sous-adresse et boucle jusqu'au paiement.

---

## Phase 1 — v0 CLI : détecter un paiement — *12–13/06/2026*

**Objectif de la session :**
Un binaire qui crée une sous-adresse de facture, l'affiche, surveille le wallet-rpc,
et ne déclare "PAYÉ" qu'une fois le montant reçu **ET** suffisamment confirmé.

**Ce que j'ai fait :**

- **Exo 1 — arguments CLI :** `account_index` et `expected_xmr` viennent maintenant de
  la ligne de commande (`cargo run -- 0 0.1`), parsés à la main depuis `env::args()`
  (pas de crate, pour apprendre).
- **Exo 2 — typage :** remplacé l'indexation `Value` de `create_address` par une struct
  `CreatedAddress { address, address_index }` peuplée via `#[derive(Deserialize)]` + `serde_json::from_value`.
- **Exo 3 — confirmations :** extraction du **minimum** des `confirmations` sur le tableau
  `in`, seuil en const, et boucle à 4 états (rien / partiel / reçu mais pas assez confirmé /
  payé+confirmé → `break`).

**Ce qui a coincé (honnête) :**

1. **const vs runtime (~2 h).** J'ai voulu mettre les arguments CLI dans des `const`.
   Ça ne compile pas : une `const` veut une valeur connue à la **compilation**, or les
   args n'existent qu'à l'**exécution**. → `let`, et parser (les args arrivent en `String`).
2. **serde, l'enveloppe.** Je ne savais pas si ma struct devait décrire toute la réponse
   JSON ou juste l'intérieur. En fait `rpc_call` a déjà retiré l'enveloppe (`result`),
   donc ma struct décrit l'objet intérieur. Confusion aussi sur "une struct ou deux"
   (→ une struct, deux champs) et sur le rôle de `derive` vs `from_value`.
3. **Option<u64> sur `min_conf`.** Voulu caster en `u8` — impossible : on ne caste pas
   un `Option`. Confusion `None` (aucun transfert) vs `Some(0)` (transfert à 0 conf).
   Et j'avais inversé la condition (déclarer payé quand les confirmations étaient basses).

**Ce que j'ai appris :**

- **`const` = compile-time, `let` = runtime.** Les args CLI sont du texte → `.parse()`,
  qui renvoie un `Result` (peut échouer) → `?`.
- **serde = pont JSON ↔ types.** `#[derive(Deserialize)]` *génère* (à la compilation)
  l'implémentation qui sait remplir MA struct ; `from_value` est le moteur générique qui
  s'en sert (à l'exécution). Les champs JSON en trop sont ignorés.
- **On ne caste pas un `Option`, on le dénoue** (`unwrap_or`, `if let Some`). `None` ≠ `Some(0)`.
  Right-sizer un entier = coller au type qu'on manipule pour éviter les casts, pas "le plus petit".
- **"parse, don't validate"** : on concentre le doute à la frontière (le parse), ensuite
  on manipule du typé sûr.
- **Politique de confirmation** : un paiement vaut sa part la moins confirmée → on prend
  le `min` ; on attend un seuil (10 en prod, 2 en test).
- Une boucle `loop` rejoue **tout** son corps à chaque tour jusqu'à `break` (ou erreur via `?`).

**Décisions techniques (et pourquoi) :**

- Parsing CLI à la main plutôt qu'un crate : comprendre avant d'abstraire.
- Structs typées plutôt que `Value` : fautes de frappe attrapées à la compilation, code
  lisible, doute levé une seule fois.
- Seuil en const `ULTIMATE_PART_CONFIRMATION: u64 = 2` (10 en prod) ; type `u64` pour
  coller à ce que renvoie serde et éviter les casts.

**Dette technique restante (à rembourser plus tard) :**

- Une seule facture à la fois ; pas de persistance (→ Phase 2).
- Les transferts (`amount`, `confirmations`) sont encore lus en `Value` brut — un
  `Vec<Transfer>` typé serait plus propre.
- Mempool / 0-conf (`"pool": true`) pas encore géré.
- Erreurs en `Box<dyn Error>` — un vrai type d'erreur viendra.

**Revue du senior :**

- ✅ Lecture de la doc API et raisonnement *avant* de coder (exo 3), décision du seuil
  avec source officielle à l'appui, et bon instinct de right-sizing — trois réflexes
  d'ingénieur. La diagnose des erreurs de compilation (types incompatibles, cast d'`Option`
  absurde) était correcte à chaque fois : le blocage venait d'un outil pas encore vu,
  pas d'un défaut de raisonnement.
- ⚠️ Mettre la struct au niveau module plutôt que dans `main` quand elle sera réutilisée.
nb : Fait le 13/06/2026
- ⚠️ `None` vs `Some(0)` : soigner le libellé des commentaires, ils décrivent des états différents.
nb : Fait le 13/06/2026

- 🧠 Timeboxing : après ~20-30 min sans progrès, changer de tactique (repro minimale,
  relire l'erreur mot à mot, demander). Les 2 h sur const/runtime étaient normales pour
  le concept ; l'important est de ne pas marteler la même approche en silence.
- 🎯 Cap : les états "partiel" et "en attente de confirmations" sont déjà les germes de
  la machine à états de la Phase 4.

**Prochaine étape :** Phase 2 — persistance (SQLite) : stocker les factures et survivre à un redémarrage.