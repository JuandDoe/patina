monero_gateway_v0
fn rpc_call(method: &str, params: Value) -> Result<Value, Box<dyn Error>>


rpc_call est une fonction libre privée définie dans main.rs. Elle n'a pas de receveur &self — ce n'est pas une méthode d'instance.
Elle prend deux paramètres :

method: &str — une référence immuable vers une chaîne de caractères représentant le nom de la méthode RPC Monero à appeler (ex: "get_balance")
params: Value — une valeur JSON passée par move (transfert de propriété) représentant les paramètres à envoyer au daemon Monero. Value est l'enum fournie par serde_json.

Elle retourne Result<Value, Box<dyn Error>> :

Ok(Value) — la réponse JSON du daemon en cas de succès
Err(Box<dyn Error>) — en cas d'échec. Box<dyn Error> est un trait object alloué sur le tas : il peut contenir n'importe quel type implémentant le trait Error (erreur réseau, erreur de parsing JSON, etc.), ce qui évite d'avoir à définir un type d'erreur spécifique

---------------------------------------------------------------------------------------------------------------------------------------------------------

serde_json::value::Value
pub fn get<I>(&self, index: I) -> Option<&Value>
where
    I: Index,
I = &str


get est une méthode d'instance publique définie sur serde_json::Value, le type enum qui représente n'importe quelle valeur JSON (objet, tableau, chaîne, nombre, booléen ou null).
Elle prend en receveur &self, une référence immuable vers la valeur courante, et un paramètre générique index: I borné par le trait Index (clause where I: Index). Ce trait est implémenté notamment par &str pour accéder à une clé d'objet JSON, et par usize pour accéder à un élément de tableau JSON.
Elle retourne Option<&Value> : Some(&Value) si la clé ou l'indice existe dans la structure, None dans les autres cas — clé absente, indice hors-bornes, ou self qui n'est ni un objet ni un tableau. C'est précisément ce qui la distingue de l'opérateur [] : là où [] panique en cas d'accès invalide, get est non-panique et force l'appelant à gérer explicitement l'absence de valeur.
La référence retournée &Value a la même durée de vie que &self (lifetime élidé par le compilateur) : impossible d'utiliser le résultat après que la valeur source a été droppée ou mutée, ce que Rust garantit statiquement.


-----------------------------------------------------------------------------------------------------------------------------------------------------------

 let response: Value = ureq::post(RPC_URL).send_json(body)?.into_json()?;


Cette ligne est une chaine de méthodes d'instance et de fonctions qui construisent une requête htpp La valeur retournée sert d'argument a l'étape suivante.
On fait une promesse au compilateur, la valeur retournée a la fin de la chaine sera de type Valeu, c'est a dire qu'il s'agira d'une représentation d'un objet json en rust.

Dans un premier temps la fonction post est appellé sur ureq. urequ est un crate rust qui met a disposition un client http minimal. La fonction n'a pas de receveur, elle a un argument qu'elle emprunte. Il s'agit d' une référence imuable de type str qiui correspond a l'url du rpc. Cette fonction post renvoie un type Request
Ensuite, la methode publique d'instance send_json est appellé sur le type Request précédement retourné, cette méthode prend deux valeur quelle consome 
un receveur self qui correspond a l'instance courante du type concret (donc de Request) et une varianle nommée data correspondant au payload de la requet, data est bornée et doit forcément implementer serde::Serialize, cela est necessaire pour permettre au payload de passer du context rust au contexte json. 
la methode renvoi un enum de type Result. Result peut prendre le type None ou Some. Si une réponse est retournée, Result encapsule un type response. Le "?" informe quant a la gestion de l'erreur eventuelle , elle est remontée a l'appellant, le programme ne panique pas.

Enfin, into_json est appellé sur Result<Response> 
Il s'agit dune méthode d'instance publique construite sur ureq::response::Response
elle contient un receiver qu'elle consume, c'est a dire qu'elle consome l'instance courante du type Response, le receiver est borné par le type DeserializeOwned puisqu'il est nécessaire que le type implémente ce train pour repasser du contexte json a la repéresntation du json en rust. Cela est egalement necessaire pour respecter la promesse fait au compilateur au départ de la chaine let response: Value

ureq
pub fn post(path: &str) -> Request

ureq::response::Response
pub fn into_json<T>(self) -> io::Result<T>
where
    T: DeserializeOwned,

pub fn send_json(self, data: impl serde::Serialize) -> Result<Response>








Cette ligne est une chaîne d'appels où chaque élément retourne un type sur lequel le suivant peut s'appuyer.
ureq::post(RPC_URL) — appel d'une fonction libre du crate ureq. Elle emprunte RPC_URL via &str et retourne un type Request représentant une requête HTTP POST non encore envoyée.
.send_json(body) — méthode appelée sur Request. Elle consomme son receveur (pas de &self, la requête ne peut plus être utilisée après) et prend body qui doit implémenter serde::Serialize — garantie que la valeur peut être sérialisée en JSON pour constituer le payload de la requête. Elle retourne Result<Response, Error>.
? — si le Result est Err, l'erreur est propagée immédiatement à l'appelant de la fonction courante. Si c'est Ok(Response), la valeur Response est extraite et la chaîne continue.
.into_json() — méthode appelée sur Response. Elle consomme la réponse HTTP et tente de désérialiser son corps en Value via serde_json. Retourne Result<Value, Error>.
? — même mécanique : propage l'erreur si la désérialisation échoue, sinon extrait le Value.
: Value — annotation de type explicite qui indique au compilateur le type attendu, nécessaire pour que serde sache vers quel type désérialiser.

-----------------------------------------------------------------------------------------------------------------------------------------------------------




















-----------------------------------------------------------------------------------------------------------------------------------------------------------

struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}

fn main() {
    let rect = Rectangle { width: 5, height: 3 };
    let area = rect.area();
    println!("Area: {}", area);
}

Quand rect.area() est appelé, Rust transforme automatiquement cet appel en Rectangle::area(&rect). rect est emprunté le temps de l'exécution de la fonction — c'est à dire que area reçoit une référence immuable vers l'instance courante rect, sans en prendre possession.
Concrètement ça implique deux choses :

Les champs de rect sont accessibles en lecture dans le corps de la fonction via self.width, self.height
rect reste valide et utilisable après l'appel, car il n'a pas été consommé
