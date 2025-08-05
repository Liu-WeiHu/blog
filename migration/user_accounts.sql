-- public.user_accounts definition
CREATE TABLE public.user_accounts (
	id int4 DEFAULT nextval('users_id_seq'::regclass) NOT NULL,
	username varchar(50) NOT NULL,
	email varchar(100) NOT NULL,
	created_at timestamp DEFAULT CURRENT_TIMESTAMP NULL,
	"password" varchar NOT NULL,
	last_login_time timestamp NULL,
	CONSTRAINT users_email_key UNIQUE (email),
	CONSTRAINT users_pkey PRIMARY KEY (id)
);
