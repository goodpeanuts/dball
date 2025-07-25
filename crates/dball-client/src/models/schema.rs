// @generated automatically by Diesel CLI.

diesel::table! {
    spot (id) {
        id -> Nullable<Integer>,
        period -> Text,
        red1 -> Integer,
        red2 -> Integer,
        red3 -> Integer,
        red4 -> Integer,
        red5 -> Integer,
        red6 -> Integer,
        blue -> Integer,
        magnification -> Integer,
        prize_status -> Nullable<Integer>,
        created_time -> Timestamp,
        modified_time -> Timestamp,
    }
}

diesel::table! {
    ticket_log (code) {
        code -> Text,
        kj_date -> Nullable<Date>,
        xq -> Nullable<Text>,
        number1 -> Nullable<Integer>,
        number2 -> Nullable<Integer>,
        number3 -> Nullable<Integer>,
        number4 -> Nullable<Integer>,
        number5 -> Nullable<Integer>,
        number6 -> Nullable<Integer>,
        number7 -> Nullable<Integer>,
        jsondata -> Nullable<Text>,
        total_sales -> Nullable<Integer>,
        jackpot -> Nullable<Integer>,
        prize1_num -> Nullable<Integer>,
        prize1_money -> Nullable<Integer>,
        prize2_num -> Nullable<Integer>,
        prize2_money -> Nullable<Integer>,
        prize3_num -> Nullable<Integer>,
        prize3_money -> Nullable<Integer>,
        prize4_num -> Nullable<Integer>,
        prize4_money -> Nullable<Integer>,
        prize5_num -> Nullable<Integer>,
        prize5_money -> Nullable<Integer>,
        prize6_num -> Nullable<Integer>,
        prize6_money -> Nullable<Integer>,
    }
}

diesel::table! {
    tickets (id) {
        id -> Nullable<Integer>,
        period -> Text,
        time -> Timestamp,
        red1 -> Integer,
        red2 -> Integer,
        red3 -> Integer,
        red4 -> Integer,
        red5 -> Integer,
        red6 -> Integer,
        blue -> Integer,
        created_time -> Timestamp,
        modified_time -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(spot, ticket_log, tickets,);
