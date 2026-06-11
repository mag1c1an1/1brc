use rand::Rng;
use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    process,
    time::Instant,
};

const MEASUREMENT_FILE: &str = "./measurements2.txt";
const STRIDE_SIZE: usize = 50_000_000;

#[derive(Clone, Copy)]
struct WeatherStation {
    id: &'static str,
    mean_temperature: f64,
}

impl WeatherStation {
    #[inline]
    fn measurement(&self, rng: &mut FastRandom, out: &mut Vec<u8>) {
        // CreateMeasurements2 不再使用真实高斯分布：
        // m = (int)mean + [-10, 10], decimal digit = [0, 9]
        let m = self.mean_temperature as i32 + rng.next_i32(21) - 10;
        let d = rng.next_i32(10) as u8;

        out.extend_from_slice(self.id.as_bytes());
        out.push(b';');
        append_i32(out, m);
        out.push(b'.');
        out.push(b'0' + d);
        out.push(b' ');
    }
}

#[derive(Clone, Copy)]
struct FastRandom {
    state: u64,
}

impl FastRandom {
    #[inline]
    fn new(seed: u64) -> Self {
        // 避免 0 seed 退化
        Self { state: seed | 1 }
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        // xorshift64*: 不是密码学随机，只是为了快速生成 benchmark 数据
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    #[inline]
    fn next_i32(&mut self, bound: i32) -> i32 {
        (self.next_u64() % bound as u64) as i32
    }
}

#[inline]
fn append_i32(out: &mut Vec<u8>, n: i32) {
    if n < 0 {
        out.push(b'-');
        append_u32(out, n.unsigned_abs());
    } else {
        append_u32(out, n as u32);
    }
}

#[inline]
fn append_u32(out: &mut Vec<u8>, mut n: u32) {
    let mut buf = [0u8; 10];
    let mut i = buf.len();

    if n == 0 {
        out.push(b'0');
        return;
    }

    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }

    out.extend_from_slice(&buf[i..]);
}

const STATIONS: &[WeatherStation] = &[
    WeatherStation {
        id: "Abha",
        mean_temperature: 18.0,
    },
    WeatherStation {
        id: "Abidjan",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Abéché",
        mean_temperature: 29.4,
    },
    WeatherStation {
        id: "Accra",
        mean_temperature: 26.4,
    },
    WeatherStation {
        id: "Addis Ababa",
        mean_temperature: 16.0,
    },
    WeatherStation {
        id: "Adelaide",
        mean_temperature: 17.3,
    },
    WeatherStation {
        id: "Aden",
        mean_temperature: 29.1,
    },
    WeatherStation {
        id: "Ahvaz",
        mean_temperature: 25.4,
    },
    WeatherStation {
        id: "Albuquerque",
        mean_temperature: 14.0,
    },
    WeatherStation {
        id: "Alexandra",
        mean_temperature: 11.0,
    },
    WeatherStation {
        id: "Alexandria",
        mean_temperature: 20.0,
    },
    WeatherStation {
        id: "Algiers",
        mean_temperature: 18.2,
    },
    WeatherStation {
        id: "Alice Springs",
        mean_temperature: 21.0,
    },
    WeatherStation {
        id: "Almaty",
        mean_temperature: 10.0,
    },
    WeatherStation {
        id: "Amsterdam",
        mean_temperature: 10.2,
    },
    WeatherStation {
        id: "Anadyr",
        mean_temperature: -6.9,
    },
    WeatherStation {
        id: "Anchorage",
        mean_temperature: 2.8,
    },
    WeatherStation {
        id: "Andorra la Vella",
        mean_temperature: 9.8,
    },
    WeatherStation {
        id: "Ankara",
        mean_temperature: 12.0,
    },
    WeatherStation {
        id: "Antananarivo",
        mean_temperature: 17.9,
    },
    WeatherStation {
        id: "Antsiranana",
        mean_temperature: 25.2,
    },
    WeatherStation {
        id: "Arkhangelsk",
        mean_temperature: 1.3,
    },
    WeatherStation {
        id: "Ashgabat",
        mean_temperature: 17.1,
    },
    WeatherStation {
        id: "Asmara",
        mean_temperature: 15.6,
    },
    WeatherStation {
        id: "Assab",
        mean_temperature: 30.5,
    },
    WeatherStation {
        id: "Astana",
        mean_temperature: 3.5,
    },
    WeatherStation {
        id: "Athens",
        mean_temperature: 19.2,
    },
    WeatherStation {
        id: "Atlanta",
        mean_temperature: 17.0,
    },
    WeatherStation {
        id: "Auckland",
        mean_temperature: 15.2,
    },
    WeatherStation {
        id: "Austin",
        mean_temperature: 20.7,
    },
    WeatherStation {
        id: "Baghdad",
        mean_temperature: 22.77,
    },
    WeatherStation {
        id: "Baguio",
        mean_temperature: 19.5,
    },
    WeatherStation {
        id: "Baku",
        mean_temperature: 15.1,
    },
    WeatherStation {
        id: "Baltimore",
        mean_temperature: 13.1,
    },
    WeatherStation {
        id: "Bamako",
        mean_temperature: 27.8,
    },
    WeatherStation {
        id: "Bangkok",
        mean_temperature: 28.6,
    },
    WeatherStation {
        id: "Bangui",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Banjul",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Barcelona",
        mean_temperature: 18.2,
    },
    WeatherStation {
        id: "Bata",
        mean_temperature: 25.1,
    },
    WeatherStation {
        id: "Batumi",
        mean_temperature: 14.0,
    },
    WeatherStation {
        id: "Beijing",
        mean_temperature: 12.9,
    },
    WeatherStation {
        id: "Beirut",
        mean_temperature: 20.9,
    },
    WeatherStation {
        id: "Belgrade",
        mean_temperature: 12.5,
    },
    WeatherStation {
        id: "Belize City",
        mean_temperature: 26.7,
    },
    WeatherStation {
        id: "Benghazi",
        mean_temperature: 19.9,
    },
    WeatherStation {
        id: "Bergen",
        mean_temperature: 7.7,
    },
    WeatherStation {
        id: "Berlin",
        mean_temperature: 10.3,
    },
    WeatherStation {
        id: "Bilbao",
        mean_temperature: 14.7,
    },
    WeatherStation {
        id: "Birao",
        mean_temperature: 26.5,
    },
    WeatherStation {
        id: "Bishkek",
        mean_temperature: 11.3,
    },
    WeatherStation {
        id: "Bissau",
        mean_temperature: 27.0,
    },
    WeatherStation {
        id: "Blantyre",
        mean_temperature: 22.2,
    },
    WeatherStation {
        id: "Bloemfontein",
        mean_temperature: 15.6,
    },
    WeatherStation {
        id: "Boise",
        mean_temperature: 11.4,
    },
    WeatherStation {
        id: "Bordeaux",
        mean_temperature: 14.2,
    },
    WeatherStation {
        id: "Bosaso",
        mean_temperature: 30.0,
    },
    WeatherStation {
        id: "Boston",
        mean_temperature: 10.9,
    },
    WeatherStation {
        id: "Bouaké",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Bratislava",
        mean_temperature: 10.5,
    },
    WeatherStation {
        id: "Brazzaville",
        mean_temperature: 25.0,
    },
    WeatherStation {
        id: "Bridgetown",
        mean_temperature: 27.0,
    },
    WeatherStation {
        id: "Brisbane",
        mean_temperature: 21.4,
    },
    WeatherStation {
        id: "Brussels",
        mean_temperature: 10.5,
    },
    WeatherStation {
        id: "Bucharest",
        mean_temperature: 10.8,
    },
    WeatherStation {
        id: "Budapest",
        mean_temperature: 11.3,
    },
    WeatherStation {
        id: "Bujumbura",
        mean_temperature: 23.8,
    },
    WeatherStation {
        id: "Bulawayo",
        mean_temperature: 18.9,
    },
    WeatherStation {
        id: "Burnie",
        mean_temperature: 13.1,
    },
    WeatherStation {
        id: "Busan",
        mean_temperature: 15.0,
    },
    WeatherStation {
        id: "Cabo San Lucas",
        mean_temperature: 23.9,
    },
    WeatherStation {
        id: "Cairns",
        mean_temperature: 25.0,
    },
    WeatherStation {
        id: "Cairo",
        mean_temperature: 21.4,
    },
    WeatherStation {
        id: "Calgary",
        mean_temperature: 4.4,
    },
    WeatherStation {
        id: "Canberra",
        mean_temperature: 13.1,
    },
    WeatherStation {
        id: "Cape Town",
        mean_temperature: 16.2,
    },
    WeatherStation {
        id: "Changsha",
        mean_temperature: 17.4,
    },
    WeatherStation {
        id: "Charlotte",
        mean_temperature: 16.1,
    },
    WeatherStation {
        id: "Chiang Mai",
        mean_temperature: 25.8,
    },
    WeatherStation {
        id: "Chicago",
        mean_temperature: 9.8,
    },
    WeatherStation {
        id: "Chihuahua",
        mean_temperature: 18.6,
    },
    WeatherStation {
        id: "Chișinău",
        mean_temperature: 10.2,
    },
    WeatherStation {
        id: "Chittagong",
        mean_temperature: 25.9,
    },
    WeatherStation {
        id: "Chongqing",
        mean_temperature: 18.6,
    },
    WeatherStation {
        id: "Christchurch",
        mean_temperature: 12.2,
    },
    WeatherStation {
        id: "City of San Marino",
        mean_temperature: 11.8,
    },
    WeatherStation {
        id: "Colombo",
        mean_temperature: 27.4,
    },
    WeatherStation {
        id: "Columbus",
        mean_temperature: 11.7,
    },
    WeatherStation {
        id: "Conakry",
        mean_temperature: 26.4,
    },
    WeatherStation {
        id: "Copenhagen",
        mean_temperature: 9.1,
    },
    WeatherStation {
        id: "Cotonou",
        mean_temperature: 27.2,
    },
    WeatherStation {
        id: "Cracow",
        mean_temperature: 9.3,
    },
    WeatherStation {
        id: "Da Lat",
        mean_temperature: 17.9,
    },
    WeatherStation {
        id: "Da Nang",
        mean_temperature: 25.8,
    },
    WeatherStation {
        id: "Dakar",
        mean_temperature: 24.0,
    },
    WeatherStation {
        id: "Dallas",
        mean_temperature: 19.0,
    },
    WeatherStation {
        id: "Damascus",
        mean_temperature: 17.0,
    },
    WeatherStation {
        id: "Dampier",
        mean_temperature: 26.4,
    },
    WeatherStation {
        id: "Dar es Salaam",
        mean_temperature: 25.8,
    },
    WeatherStation {
        id: "Darwin",
        mean_temperature: 27.6,
    },
    WeatherStation {
        id: "Denpasar",
        mean_temperature: 23.7,
    },
    WeatherStation {
        id: "Denver",
        mean_temperature: 10.4,
    },
    WeatherStation {
        id: "Detroit",
        mean_temperature: 10.0,
    },
    WeatherStation {
        id: "Dhaka",
        mean_temperature: 25.9,
    },
    WeatherStation {
        id: "Dikson",
        mean_temperature: -11.1,
    },
    WeatherStation {
        id: "Dili",
        mean_temperature: 26.6,
    },
    WeatherStation {
        id: "Djibouti",
        mean_temperature: 29.9,
    },
    WeatherStation {
        id: "Dodoma",
        mean_temperature: 22.7,
    },
    WeatherStation {
        id: "Dolisie",
        mean_temperature: 24.0,
    },
    WeatherStation {
        id: "Douala",
        mean_temperature: 26.7,
    },
    WeatherStation {
        id: "Dubai",
        mean_temperature: 26.9,
    },
    WeatherStation {
        id: "Dublin",
        mean_temperature: 9.8,
    },
    WeatherStation {
        id: "Dunedin",
        mean_temperature: 11.1,
    },
    WeatherStation {
        id: "Durban",
        mean_temperature: 20.6,
    },
    WeatherStation {
        id: "Dushanbe",
        mean_temperature: 14.7,
    },
    WeatherStation {
        id: "Edinburgh",
        mean_temperature: 9.3,
    },
    WeatherStation {
        id: "Edmonton",
        mean_temperature: 4.2,
    },
    WeatherStation {
        id: "El Paso",
        mean_temperature: 18.1,
    },
    WeatherStation {
        id: "Entebbe",
        mean_temperature: 21.0,
    },
    WeatherStation {
        id: "Erbil",
        mean_temperature: 19.5,
    },
    WeatherStation {
        id: "Erzurum",
        mean_temperature: 5.1,
    },
    WeatherStation {
        id: "Fairbanks",
        mean_temperature: -2.3,
    },
    WeatherStation {
        id: "Fianarantsoa",
        mean_temperature: 17.9,
    },
    WeatherStation {
        id: "Flores,  Petén",
        mean_temperature: 26.4,
    },
    WeatherStation {
        id: "Frankfurt",
        mean_temperature: 10.6,
    },
    WeatherStation {
        id: "Fresno",
        mean_temperature: 17.9,
    },
    WeatherStation {
        id: "Fukuoka",
        mean_temperature: 17.0,
    },
    WeatherStation {
        id: "Gabès",
        mean_temperature: 19.5,
    },
    WeatherStation {
        id: "Gaborone",
        mean_temperature: 21.0,
    },
    WeatherStation {
        id: "Gagnoa",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Gangtok",
        mean_temperature: 15.2,
    },
    WeatherStation {
        id: "Garissa",
        mean_temperature: 29.3,
    },
    WeatherStation {
        id: "Garoua",
        mean_temperature: 28.3,
    },
    WeatherStation {
        id: "George Town",
        mean_temperature: 27.9,
    },
    WeatherStation {
        id: "Ghanzi",
        mean_temperature: 21.4,
    },
    WeatherStation {
        id: "Gjoa Haven",
        mean_temperature: -14.4,
    },
    WeatherStation {
        id: "Guadalajara",
        mean_temperature: 20.9,
    },
    WeatherStation {
        id: "Guangzhou",
        mean_temperature: 22.4,
    },
    WeatherStation {
        id: "Guatemala City",
        mean_temperature: 20.4,
    },
    WeatherStation {
        id: "Halifax",
        mean_temperature: 7.5,
    },
    WeatherStation {
        id: "Hamburg",
        mean_temperature: 9.7,
    },
    WeatherStation {
        id: "Hamilton",
        mean_temperature: 13.8,
    },
    WeatherStation {
        id: "Hanga Roa",
        mean_temperature: 20.5,
    },
    WeatherStation {
        id: "Hanoi",
        mean_temperature: 23.6,
    },
    WeatherStation {
        id: "Harare",
        mean_temperature: 18.4,
    },
    WeatherStation {
        id: "Harbin",
        mean_temperature: 5.0,
    },
    WeatherStation {
        id: "Hargeisa",
        mean_temperature: 21.7,
    },
    WeatherStation {
        id: "Hat Yai",
        mean_temperature: 27.0,
    },
    WeatherStation {
        id: "Havana",
        mean_temperature: 25.2,
    },
    WeatherStation {
        id: "Helsinki",
        mean_temperature: 5.9,
    },
    WeatherStation {
        id: "Heraklion",
        mean_temperature: 18.9,
    },
    WeatherStation {
        id: "Hiroshima",
        mean_temperature: 16.3,
    },
    WeatherStation {
        id: "Ho Chi Minh City",
        mean_temperature: 27.4,
    },
    WeatherStation {
        id: "Hobart",
        mean_temperature: 12.7,
    },
    WeatherStation {
        id: "Hong Kong",
        mean_temperature: 23.3,
    },
    WeatherStation {
        id: "Honiara",
        mean_temperature: 26.5,
    },
    WeatherStation {
        id: "Honolulu",
        mean_temperature: 25.4,
    },
    WeatherStation {
        id: "Houston",
        mean_temperature: 20.8,
    },
    WeatherStation {
        id: "Ifrane",
        mean_temperature: 11.4,
    },
    WeatherStation {
        id: "Indianapolis",
        mean_temperature: 11.8,
    },
    WeatherStation {
        id: "Iqaluit",
        mean_temperature: -9.3,
    },
    WeatherStation {
        id: "Irkutsk",
        mean_temperature: 1.0,
    },
    WeatherStation {
        id: "Istanbul",
        mean_temperature: 13.9,
    },
    WeatherStation {
        id: "İzmir",
        mean_temperature: 17.9,
    },
    WeatherStation {
        id: "Jacksonville",
        mean_temperature: 20.3,
    },
    WeatherStation {
        id: "Jakarta",
        mean_temperature: 26.7,
    },
    WeatherStation {
        id: "Jayapura",
        mean_temperature: 27.0,
    },
    WeatherStation {
        id: "Jerusalem",
        mean_temperature: 18.3,
    },
    WeatherStation {
        id: "Johannesburg",
        mean_temperature: 15.5,
    },
    WeatherStation {
        id: "Jos",
        mean_temperature: 22.8,
    },
    WeatherStation {
        id: "Juba",
        mean_temperature: 27.8,
    },
    WeatherStation {
        id: "Kabul",
        mean_temperature: 12.1,
    },
    WeatherStation {
        id: "Kampala",
        mean_temperature: 20.0,
    },
    WeatherStation {
        id: "Kandi",
        mean_temperature: 27.7,
    },
    WeatherStation {
        id: "Kankan",
        mean_temperature: 26.5,
    },
    WeatherStation {
        id: "Kano",
        mean_temperature: 26.4,
    },
    WeatherStation {
        id: "Kansas City",
        mean_temperature: 12.5,
    },
    WeatherStation {
        id: "Karachi",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Karonga",
        mean_temperature: 24.4,
    },
    WeatherStation {
        id: "Kathmandu",
        mean_temperature: 18.3,
    },
    WeatherStation {
        id: "Khartoum",
        mean_temperature: 29.9,
    },
    WeatherStation {
        id: "Kingston",
        mean_temperature: 27.4,
    },
    WeatherStation {
        id: "Kinshasa",
        mean_temperature: 25.3,
    },
    WeatherStation {
        id: "Kolkata",
        mean_temperature: 26.7,
    },
    WeatherStation {
        id: "Kuala Lumpur",
        mean_temperature: 27.3,
    },
    WeatherStation {
        id: "Kumasi",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Kunming",
        mean_temperature: 15.7,
    },
    WeatherStation {
        id: "Kuopio",
        mean_temperature: 3.4,
    },
    WeatherStation {
        id: "Kuwait City",
        mean_temperature: 25.7,
    },
    WeatherStation {
        id: "Kyiv",
        mean_temperature: 8.4,
    },
    WeatherStation {
        id: "Kyoto",
        mean_temperature: 15.8,
    },
    WeatherStation {
        id: "La Ceiba",
        mean_temperature: 26.2,
    },
    WeatherStation {
        id: "La Paz",
        mean_temperature: 23.7,
    },
    WeatherStation {
        id: "Lagos",
        mean_temperature: 26.8,
    },
    WeatherStation {
        id: "Lahore",
        mean_temperature: 24.3,
    },
    WeatherStation {
        id: "Lake Havasu City",
        mean_temperature: 23.7,
    },
    WeatherStation {
        id: "Lake Tekapo",
        mean_temperature: 8.7,
    },
    WeatherStation {
        id: "Las Palmas de Gran Canaria",
        mean_temperature: 21.2,
    },
    WeatherStation {
        id: "Las Vegas",
        mean_temperature: 20.3,
    },
    WeatherStation {
        id: "Launceston",
        mean_temperature: 13.1,
    },
    WeatherStation {
        id: "Lhasa",
        mean_temperature: 7.6,
    },
    WeatherStation {
        id: "Libreville",
        mean_temperature: 25.9,
    },
    WeatherStation {
        id: "Lisbon",
        mean_temperature: 17.5,
    },
    WeatherStation {
        id: "Livingstone",
        mean_temperature: 21.8,
    },
    WeatherStation {
        id: "Ljubljana",
        mean_temperature: 10.9,
    },
    WeatherStation {
        id: "Lodwar",
        mean_temperature: 29.3,
    },
    WeatherStation {
        id: "Lomé",
        mean_temperature: 26.9,
    },
    WeatherStation {
        id: "London",
        mean_temperature: 11.3,
    },
    WeatherStation {
        id: "Los Angeles",
        mean_temperature: 18.6,
    },
    WeatherStation {
        id: "Louisville",
        mean_temperature: 13.9,
    },
    WeatherStation {
        id: "Luanda",
        mean_temperature: 25.8,
    },
    WeatherStation {
        id: "Lubumbashi",
        mean_temperature: 20.8,
    },
    WeatherStation {
        id: "Lusaka",
        mean_temperature: 19.9,
    },
    WeatherStation {
        id: "Luxembourg City",
        mean_temperature: 9.3,
    },
    WeatherStation {
        id: "Lviv",
        mean_temperature: 7.8,
    },
    WeatherStation {
        id: "Lyon",
        mean_temperature: 12.5,
    },
    WeatherStation {
        id: "Madrid",
        mean_temperature: 15.0,
    },
    WeatherStation {
        id: "Mahajanga",
        mean_temperature: 26.3,
    },
    WeatherStation {
        id: "Makassar",
        mean_temperature: 26.7,
    },
    WeatherStation {
        id: "Makurdi",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Malabo",
        mean_temperature: 26.3,
    },
    WeatherStation {
        id: "Malé",
        mean_temperature: 28.0,
    },
    WeatherStation {
        id: "Managua",
        mean_temperature: 27.3,
    },
    WeatherStation {
        id: "Manama",
        mean_temperature: 26.5,
    },
    WeatherStation {
        id: "Mandalay",
        mean_temperature: 28.0,
    },
    WeatherStation {
        id: "Mango",
        mean_temperature: 28.1,
    },
    WeatherStation {
        id: "Manila",
        mean_temperature: 28.4,
    },
    WeatherStation {
        id: "Maputo",
        mean_temperature: 22.8,
    },
    WeatherStation {
        id: "Marrakesh",
        mean_temperature: 19.6,
    },
    WeatherStation {
        id: "Marseille",
        mean_temperature: 15.8,
    },
    WeatherStation {
        id: "Maun",
        mean_temperature: 22.4,
    },
    WeatherStation {
        id: "Medan",
        mean_temperature: 26.5,
    },
    WeatherStation {
        id: "Mek'ele",
        mean_temperature: 22.7,
    },
    WeatherStation {
        id: "Melbourne",
        mean_temperature: 15.1,
    },
    WeatherStation {
        id: "Memphis",
        mean_temperature: 17.2,
    },
    WeatherStation {
        id: "Mexicali",
        mean_temperature: 23.1,
    },
    WeatherStation {
        id: "Mexico City",
        mean_temperature: 17.5,
    },
    WeatherStation {
        id: "Miami",
        mean_temperature: 24.9,
    },
    WeatherStation {
        id: "Milan",
        mean_temperature: 13.0,
    },
    WeatherStation {
        id: "Milwaukee",
        mean_temperature: 8.9,
    },
    WeatherStation {
        id: "Minneapolis",
        mean_temperature: 7.8,
    },
    WeatherStation {
        id: "Minsk",
        mean_temperature: 6.7,
    },
    WeatherStation {
        id: "Mogadishu",
        mean_temperature: 27.1,
    },
    WeatherStation {
        id: "Mombasa",
        mean_temperature: 26.3,
    },
    WeatherStation {
        id: "Monaco",
        mean_temperature: 16.4,
    },
    WeatherStation {
        id: "Moncton",
        mean_temperature: 6.1,
    },
    WeatherStation {
        id: "Monterrey",
        mean_temperature: 22.3,
    },
    WeatherStation {
        id: "Montreal",
        mean_temperature: 6.8,
    },
    WeatherStation {
        id: "Moscow",
        mean_temperature: 5.8,
    },
    WeatherStation {
        id: "Mumbai",
        mean_temperature: 27.1,
    },
    WeatherStation {
        id: "Murmansk",
        mean_temperature: 0.6,
    },
    WeatherStation {
        id: "Muscat",
        mean_temperature: 28.0,
    },
    WeatherStation {
        id: "Mzuzu",
        mean_temperature: 17.7,
    },
    WeatherStation {
        id: "N'Djamena",
        mean_temperature: 28.3,
    },
    WeatherStation {
        id: "Naha",
        mean_temperature: 23.1,
    },
    WeatherStation {
        id: "Nairobi",
        mean_temperature: 17.8,
    },
    WeatherStation {
        id: "Nakhon Ratchasima",
        mean_temperature: 27.3,
    },
    WeatherStation {
        id: "Napier",
        mean_temperature: 14.6,
    },
    WeatherStation {
        id: "Napoli",
        mean_temperature: 15.9,
    },
    WeatherStation {
        id: "Nashville",
        mean_temperature: 15.4,
    },
    WeatherStation {
        id: "Nassau",
        mean_temperature: 24.6,
    },
    WeatherStation {
        id: "Ndola",
        mean_temperature: 20.3,
    },
    WeatherStation {
        id: "New Delhi",
        mean_temperature: 25.0,
    },
    WeatherStation {
        id: "New Orleans",
        mean_temperature: 20.7,
    },
    WeatherStation {
        id: "New York City",
        mean_temperature: 12.9,
    },
    WeatherStation {
        id: "Ngaoundéré",
        mean_temperature: 22.0,
    },
    WeatherStation {
        id: "Niamey",
        mean_temperature: 29.3,
    },
    WeatherStation {
        id: "Nicosia",
        mean_temperature: 19.7,
    },
    WeatherStation {
        id: "Niigata",
        mean_temperature: 13.9,
    },
    WeatherStation {
        id: "Nouadhibou",
        mean_temperature: 21.3,
    },
    WeatherStation {
        id: "Nouakchott",
        mean_temperature: 25.7,
    },
    WeatherStation {
        id: "Novosibirsk",
        mean_temperature: 1.7,
    },
    WeatherStation {
        id: "Nuuk",
        mean_temperature: -1.4,
    },
    WeatherStation {
        id: "Odesa",
        mean_temperature: 10.7,
    },
    WeatherStation {
        id: "Odienné",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Oklahoma City",
        mean_temperature: 15.9,
    },
    WeatherStation {
        id: "Omaha",
        mean_temperature: 10.6,
    },
    WeatherStation {
        id: "Oranjestad",
        mean_temperature: 28.1,
    },
    WeatherStation {
        id: "Oslo",
        mean_temperature: 5.7,
    },
    WeatherStation {
        id: "Ottawa",
        mean_temperature: 6.6,
    },
    WeatherStation {
        id: "Ouagadougou",
        mean_temperature: 28.3,
    },
    WeatherStation {
        id: "Ouahigouya",
        mean_temperature: 28.6,
    },
    WeatherStation {
        id: "Ouarzazate",
        mean_temperature: 18.9,
    },
    WeatherStation {
        id: "Oulu",
        mean_temperature: 2.7,
    },
    WeatherStation {
        id: "Palembang",
        mean_temperature: 27.3,
    },
    WeatherStation {
        id: "Palermo",
        mean_temperature: 18.5,
    },
    WeatherStation {
        id: "Palm Springs",
        mean_temperature: 24.5,
    },
    WeatherStation {
        id: "Palmerston North",
        mean_temperature: 13.2,
    },
    WeatherStation {
        id: "Panama City",
        mean_temperature: 28.0,
    },
    WeatherStation {
        id: "Parakou",
        mean_temperature: 26.8,
    },
    WeatherStation {
        id: "Paris",
        mean_temperature: 12.3,
    },
    WeatherStation {
        id: "Perth",
        mean_temperature: 18.7,
    },
    WeatherStation {
        id: "Petropavlovsk-Kamchatsky",
        mean_temperature: 1.9,
    },
    WeatherStation {
        id: "Philadelphia",
        mean_temperature: 13.2,
    },
    WeatherStation {
        id: "Phnom Penh",
        mean_temperature: 28.3,
    },
    WeatherStation {
        id: "Phoenix",
        mean_temperature: 23.9,
    },
    WeatherStation {
        id: "Pittsburgh",
        mean_temperature: 10.8,
    },
    WeatherStation {
        id: "Podgorica",
        mean_temperature: 15.3,
    },
    WeatherStation {
        id: "Pointe-Noire",
        mean_temperature: 26.1,
    },
    WeatherStation {
        id: "Pontianak",
        mean_temperature: 27.7,
    },
    WeatherStation {
        id: "Port Moresby",
        mean_temperature: 26.9,
    },
    WeatherStation {
        id: "Port Sudan",
        mean_temperature: 28.4,
    },
    WeatherStation {
        id: "Port Vila",
        mean_temperature: 24.3,
    },
    WeatherStation {
        id: "Port-Gentil",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Portland (OR)",
        mean_temperature: 12.4,
    },
    WeatherStation {
        id: "Porto",
        mean_temperature: 15.7,
    },
    WeatherStation {
        id: "Prague",
        mean_temperature: 8.4,
    },
    WeatherStation {
        id: "Praia",
        mean_temperature: 24.4,
    },
    WeatherStation {
        id: "Pretoria",
        mean_temperature: 18.2,
    },
    WeatherStation {
        id: "Pyongyang",
        mean_temperature: 10.8,
    },
    WeatherStation {
        id: "Rabat",
        mean_temperature: 17.2,
    },
    WeatherStation {
        id: "Rangpur",
        mean_temperature: 24.4,
    },
    WeatherStation {
        id: "Reggane",
        mean_temperature: 28.3,
    },
    WeatherStation {
        id: "Reykjavík",
        mean_temperature: 4.3,
    },
    WeatherStation {
        id: "Riga",
        mean_temperature: 6.2,
    },
    WeatherStation {
        id: "Riyadh",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Rome",
        mean_temperature: 15.2,
    },
    WeatherStation {
        id: "Roseau",
        mean_temperature: 26.2,
    },
    WeatherStation {
        id: "Rostov-on-Don",
        mean_temperature: 9.9,
    },
    WeatherStation {
        id: "Sacramento",
        mean_temperature: 16.3,
    },
    WeatherStation {
        id: "Saint Petersburg",
        mean_temperature: 5.8,
    },
    WeatherStation {
        id: "Saint-Pierre",
        mean_temperature: 5.7,
    },
    WeatherStation {
        id: "Salt Lake City",
        mean_temperature: 11.6,
    },
    WeatherStation {
        id: "San Antonio",
        mean_temperature: 20.8,
    },
    WeatherStation {
        id: "San Diego",
        mean_temperature: 17.8,
    },
    WeatherStation {
        id: "San Francisco",
        mean_temperature: 14.6,
    },
    WeatherStation {
        id: "San Jose",
        mean_temperature: 16.4,
    },
    WeatherStation {
        id: "San José",
        mean_temperature: 22.6,
    },
    WeatherStation {
        id: "San Juan",
        mean_temperature: 27.2,
    },
    WeatherStation {
        id: "San Salvador",
        mean_temperature: 23.1,
    },
    WeatherStation {
        id: "Sana'a",
        mean_temperature: 20.0,
    },
    WeatherStation {
        id: "Santo Domingo",
        mean_temperature: 25.9,
    },
    WeatherStation {
        id: "Sapporo",
        mean_temperature: 8.9,
    },
    WeatherStation {
        id: "Sarajevo",
        mean_temperature: 10.1,
    },
    WeatherStation {
        id: "Saskatoon",
        mean_temperature: 3.3,
    },
    WeatherStation {
        id: "Seattle",
        mean_temperature: 11.3,
    },
    WeatherStation {
        id: "Ségou",
        mean_temperature: 28.0,
    },
    WeatherStation {
        id: "Seoul",
        mean_temperature: 12.5,
    },
    WeatherStation {
        id: "Seville",
        mean_temperature: 19.2,
    },
    WeatherStation {
        id: "Shanghai",
        mean_temperature: 16.7,
    },
    WeatherStation {
        id: "Singapore",
        mean_temperature: 27.0,
    },
    WeatherStation {
        id: "Skopje",
        mean_temperature: 12.4,
    },
    WeatherStation {
        id: "Sochi",
        mean_temperature: 14.2,
    },
    WeatherStation {
        id: "Sofia",
        mean_temperature: 10.6,
    },
    WeatherStation {
        id: "Sokoto",
        mean_temperature: 28.0,
    },
    WeatherStation {
        id: "Split",
        mean_temperature: 16.1,
    },
    WeatherStation {
        id: "St. John's",
        mean_temperature: 5.0,
    },
    WeatherStation {
        id: "St. Louis",
        mean_temperature: 13.9,
    },
    WeatherStation {
        id: "Stockholm",
        mean_temperature: 6.6,
    },
    WeatherStation {
        id: "Surabaya",
        mean_temperature: 27.1,
    },
    WeatherStation {
        id: "Suva",
        mean_temperature: 25.6,
    },
    WeatherStation {
        id: "Suwałki",
        mean_temperature: 7.2,
    },
    WeatherStation {
        id: "Sydney",
        mean_temperature: 17.7,
    },
    WeatherStation {
        id: "Tabora",
        mean_temperature: 23.0,
    },
    WeatherStation {
        id: "Tabriz",
        mean_temperature: 12.6,
    },
    WeatherStation {
        id: "Taipei",
        mean_temperature: 23.0,
    },
    WeatherStation {
        id: "Tallinn",
        mean_temperature: 6.4,
    },
    WeatherStation {
        id: "Tamale",
        mean_temperature: 27.9,
    },
    WeatherStation {
        id: "Tamanrasset",
        mean_temperature: 21.7,
    },
    WeatherStation {
        id: "Tampa",
        mean_temperature: 22.9,
    },
    WeatherStation {
        id: "Tashkent",
        mean_temperature: 14.8,
    },
    WeatherStation {
        id: "Tauranga",
        mean_temperature: 14.8,
    },
    WeatherStation {
        id: "Tbilisi",
        mean_temperature: 12.9,
    },
    WeatherStation {
        id: "Tegucigalpa",
        mean_temperature: 21.7,
    },
    WeatherStation {
        id: "Tehran",
        mean_temperature: 17.0,
    },
    WeatherStation {
        id: "Tel Aviv",
        mean_temperature: 20.0,
    },
    WeatherStation {
        id: "Thessaloniki",
        mean_temperature: 16.0,
    },
    WeatherStation {
        id: "Thiès",
        mean_temperature: 24.0,
    },
    WeatherStation {
        id: "Tijuana",
        mean_temperature: 17.8,
    },
    WeatherStation {
        id: "Timbuktu",
        mean_temperature: 28.0,
    },
    WeatherStation {
        id: "Tirana",
        mean_temperature: 15.2,
    },
    WeatherStation {
        id: "Toamasina",
        mean_temperature: 23.4,
    },
    WeatherStation {
        id: "Tokyo",
        mean_temperature: 15.4,
    },
    WeatherStation {
        id: "Toliara",
        mean_temperature: 24.1,
    },
    WeatherStation {
        id: "Toluca",
        mean_temperature: 12.4,
    },
    WeatherStation {
        id: "Toronto",
        mean_temperature: 9.4,
    },
    WeatherStation {
        id: "Tripoli",
        mean_temperature: 20.0,
    },
    WeatherStation {
        id: "Tromsø",
        mean_temperature: 2.9,
    },
    WeatherStation {
        id: "Tucson",
        mean_temperature: 20.9,
    },
    WeatherStation {
        id: "Tunis",
        mean_temperature: 18.4,
    },
    WeatherStation {
        id: "Ulaanbaatar",
        mean_temperature: -0.4,
    },
    WeatherStation {
        id: "Upington",
        mean_temperature: 20.4,
    },
    WeatherStation {
        id: "Ürümqi",
        mean_temperature: 7.4,
    },
    WeatherStation {
        id: "Vaduz",
        mean_temperature: 10.1,
    },
    WeatherStation {
        id: "Valencia",
        mean_temperature: 18.3,
    },
    WeatherStation {
        id: "Valletta",
        mean_temperature: 18.8,
    },
    WeatherStation {
        id: "Vancouver",
        mean_temperature: 10.4,
    },
    WeatherStation {
        id: "Veracruz",
        mean_temperature: 25.4,
    },
    WeatherStation {
        id: "Vienna",
        mean_temperature: 10.4,
    },
    WeatherStation {
        id: "Vientiane",
        mean_temperature: 25.9,
    },
    WeatherStation {
        id: "Villahermosa",
        mean_temperature: 27.1,
    },
    WeatherStation {
        id: "Vilnius",
        mean_temperature: 6.0,
    },
    WeatherStation {
        id: "Virginia Beach",
        mean_temperature: 15.8,
    },
    WeatherStation {
        id: "Vladivostok",
        mean_temperature: 4.9,
    },
    WeatherStation {
        id: "Warsaw",
        mean_temperature: 8.5,
    },
    WeatherStation {
        id: "Washington, D.C.",
        mean_temperature: 14.6,
    },
    WeatherStation {
        id: "Wau",
        mean_temperature: 27.8,
    },
    WeatherStation {
        id: "Wellington",
        mean_temperature: 12.9,
    },
    WeatherStation {
        id: "Whitehorse",
        mean_temperature: -0.1,
    },
    WeatherStation {
        id: "Wichita",
        mean_temperature: 13.9,
    },
    WeatherStation {
        id: "Willemstad",
        mean_temperature: 28.0,
    },
    WeatherStation {
        id: "Winnipeg",
        mean_temperature: 3.0,
    },
    WeatherStation {
        id: "Wrocław",
        mean_temperature: 9.6,
    },
    WeatherStation {
        id: "Xi'an",
        mean_temperature: 14.1,
    },
    WeatherStation {
        id: "Yakutsk",
        mean_temperature: -8.8,
    },
    WeatherStation {
        id: "Yangon",
        mean_temperature: 27.5,
    },
    WeatherStation {
        id: "Yaoundé",
        mean_temperature: 23.8,
    },
    WeatherStation {
        id: "Yellowknife",
        mean_temperature: -4.3,
    },
    WeatherStation {
        id: "Yerevan",
        mean_temperature: 12.4,
    },
    WeatherStation {
        id: "Yinchuan",
        mean_temperature: 9.0,
    },
    WeatherStation {
        id: "Zagreb",
        mean_temperature: 10.7,
    },
    WeatherStation {
        id: "Zanzibar City",
        mean_temperature: 26.0,
    },
    WeatherStation {
        id: "Zürich",
        mean_temperature: 9.3,
    },
];

fn main() -> std::io::Result<()> {
    let start = Instant::now();

    let Some(arg) = env::args().nth(1) else {
        eprintln!("Usage: create_measurements.sh <number of records to create>");
        process::exit(1);
    };

    let size: usize = match arg.parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid value for <number of records to create>");
            eprintln!("Usage: CreateMeasurements <number of records to create>");
            process::exit(1);
        }
    };

    let file = File::create(MEASUREMENT_FILE)?;
    let mut writer = BufWriter::with_capacity(16 * 1024 * 1024, file);

    let outer = size / STRIDE_SIZE;
    let remainder = size - outer * STRIDE_SIZE;

    for i in 0..outer {
        produce(&mut writer, STRIDE_SIZE)?;
        println!(
            "Wrote {} measurements in {} ms",
            (i + 1) * STRIDE_SIZE,
            start.elapsed().as_millis()
        );
    }

    produce(&mut writer, remainder)?;
    writer.flush()?;

    println!(
        "Created file with {} measurements in {} ms",
        size,
        start.elapsed().as_millis()
    );

    Ok(())
}

fn produce<W: Write>(writer: &mut W, count: usize) -> std::io::Result<()> {
    let station_count = STATIONS.len();
    let rest = count % 8;
    let main_count = count - rest;

    let mut seed_rng = rand::rng();
    let mut r1 = FastRandom::new(seed_rng.next_u64());
    let mut r2 = FastRandom::new(seed_rng.next_u64());
    let mut r3 = FastRandom::new(seed_rng.next_u64());
    let mut r4 = FastRandom::new(seed_rng.next_u64());

    // 对应 Java 里的 CheaperCharBuffer：重复使用固定 buffer，避免每行 String 分配。
    let mut buf = Vec::with_capacity(1024);

    for _ in (0..main_count).step_by(8) {
        let s1 = r1.next_i32(station_count as i32) as usize;
        let s2 = r2.next_i32(station_count as i32) as usize;
        let s3 = r3.next_i32(station_count as i32) as usize;
        let s4 = r4.next_i32(station_count as i32) as usize;
        STATIONS[s1].measurement(&mut r1, &mut buf);
        STATIONS[s2].measurement(&mut r2, &mut buf);
        STATIONS[s3].measurement(&mut r3, &mut buf);
        STATIONS[s4].measurement(&mut r4, &mut buf);

        let s1 = r1.next_i32(station_count as i32) as usize;
        let s2 = r2.next_i32(station_count as i32) as usize;
        let s3 = r3.next_i32(station_count as i32) as usize;
        let s4 = r4.next_i32(station_count as i32) as usize;
        STATIONS[s1].measurement(&mut r1, &mut buf);
        STATIONS[s2].measurement(&mut r2, &mut buf);
        STATIONS[s3].measurement(&mut r3, &mut buf);
        STATIONS[s4].measurement(&mut r4, &mut buf);

        writer.write_all(&buf)?;
        buf.clear();
    }

    for _ in 0..rest {
        let s = r1.next_i32(station_count as i32) as usize;
        STATIONS[s].measurement(&mut r1, &mut buf);
        writer.write_all(&buf)?;
        buf.clear();
    }

    Ok(())
}
