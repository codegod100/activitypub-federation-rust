module default {
	type Person {
		required property username -> str{
			constraint exclusive;
		}
		multi property links -> tuple<rel: str, type: str, href: str>;
	}
}
