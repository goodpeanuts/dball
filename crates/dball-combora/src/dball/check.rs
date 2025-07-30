use crate::dball::{DBall, Reward};

impl DBall {
    /// Check prize level
    ///
    /// # Parameters
    /// * `ticket` - Ticket numbers
    /// * `winning_ticket` - Winning numbers
    ///
    /// # Returns
    /// Returns the prize level
    pub fn check_prize(&self, winning_ticket: &Self) -> Reward {
        // Count red ball matches
        let red_matches = self
            .rball
            .iter()
            .filter(|&r| winning_ticket.rball.contains(r))
            .count();

        // Check if blue ball matches
        let blue_matches = self.bball == winning_ticket.bball;

        // Determine prize based on matches
        match (red_matches, blue_matches) {
            (6, true) => Reward::FirstPrize,
            (6, false) => Reward::SecondPrize,
            (5, true) => Reward::ThirdPrize,
            (5, false) | (4, true) => Reward::FourthPrize,
            (4, false) | (3, true) => Reward::FifthPrize,
            (_, true) => Reward::SixthPrize,
            _ => Reward::NoWin,
        }
    }

    /// Check multiple tickets against a winning ticket
    pub fn check_multiple_tickets(tickets: &[Self], winning_ticket: &Self) -> Vec<(usize, Reward)> {
        tickets
            .iter()
            .enumerate()
            .map(|(index, ticket)| (index, ticket.check_prize(winning_ticket)))
            .collect()
    }

    #[deprecated(note = "price not precise")]
    /// Calculate total prize amount for multiple tickets
    pub fn calculate_total_prize(tickets: &[Self], winning_ticket: &Self) -> u32 {
        tickets
            .iter()
            .map(|ticket| ticket.check_prize(winning_ticket).prize_amount())
            .sum()
    }

    /// Count prizes by type for multiple tickets
    pub fn count_prizes_list(
        tickets: &[Self],
        winning_ticket: &Self,
    ) -> std::collections::HashMap<String, usize> {
        let mut counts = std::collections::HashMap::new();

        for ticket in tickets {
            let reward = ticket.check_prize(winning_ticket);
            let key = reward.description().to_owned();
            *counts.entry(key).or_insert(0) += 1;
        }

        counts
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::bluemorn::BlueMorn;

    use super::*;

    fn create_test_ticket(rball: [u8; 6], bball: u8) -> DBall {
        let mut rball = rball;
        DBall::new_one(&mut rball[..], bball).unwrap()
    }

    #[test]
    fn test_first_prize() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::FirstPrize);
        assert_eq!(result.prize_amount(), 4_500_000);
    }

    #[test]
    fn test_second_prize() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 8);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::SecondPrize);
        assert_eq!(result.prize_amount(), 150_000);
    }

    #[test]
    fn test_third_prize() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([1, 2, 3, 4, 5, 10], 7);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::ThirdPrize);
        assert_eq!(result.prize_amount(), 3_000);
    }

    #[test]
    fn test_fourth_prize_5_red() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([1, 2, 3, 4, 5, 10], 8);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::FourthPrize);
        assert_eq!(result.prize_amount(), 200);
    }

    #[test]
    fn test_fourth_prize_4_red_1_blue() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([1, 2, 3, 4, 10, 11], 7);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::FourthPrize);
        assert_eq!(result.prize_amount(), 200);
    }

    #[test]
    fn test_fifth_prize_4_red() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([1, 2, 3, 4, 10, 11], 8);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::FifthPrize);
        assert_eq!(result.prize_amount(), 10);
    }

    #[test]
    fn test_fifth_prize_3_red_1_blue() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([1, 2, 3, 10, 11, 12], 7);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::FifthPrize);
        assert_eq!(result.prize_amount(), 10);
    }

    #[test]
    fn test_sixth_prize() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([10, 11, 12, 13, 14, 15], 7);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::SixthPrize);
        assert_eq!(result.prize_amount(), 5);
    }

    #[test]
    fn test_no_win() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let ticket = create_test_ticket([10, 11, 12, 13, 14, 15], 8);

        let result = ticket.check_prize(&winning_ticket);
        assert_eq!(result, Reward::NoWin);
        assert_eq!(result.prize_amount(), 0);
    }

    #[test]
    fn test_ticket_creation_valid() {
        let mut rball = [1, 2, 3, 4, 5, 6];
        let ticket = DBall::new_one(&mut rball[..], 7);
        assert!(ticket.is_ok());

        let ticket = ticket.unwrap();
        assert_eq!(ticket.rball.len(), 6);
        assert_eq!(ticket.bball, 7);
    }

    #[test]
    fn test_ticket_creation_invalid_red_count() {
        // Test with 5 red balls instead of 6
        let mut rball = [1, 2, 3, 4, 5];
        let ticket = DBall::new_one(&mut rball[..], 7);
        assert!(ticket.is_err());

        // Test with 7 red balls instead of 6
        let mut rball = [1, 2, 3, 4, 5, 6, 7];
        let ticket = DBall::new_one(&mut rball[..], 7);
        assert!(ticket.is_err());
    }

    #[test]
    fn test_ticket_creation_invalid_red_range() {
        let mut rball = [1, 2, 3, 4, 5, 34];
        let ticket = DBall::new_one(&mut rball[..], 7);
        assert!(ticket.is_err());
    }

    #[test]
    fn test_ticket_creation_duplicate_red() {
        let mut rball = [1, 2, 3, 4, 5, 5];
        let ticket = DBall::new_one(&mut rball[..], 7);
        assert!(ticket.is_err());
    }

    #[test]
    fn test_ticket_creation_invalid_blue_range() {
        let mut rball = [1, 2, 3, 4, 5, 6];
        let ticket = DBall::new_one(&mut rball[..], 17);
        assert!(ticket.is_err());
    }

    #[test]
    fn test_generator_with_seed() {
        // Using fixed seed should produce repeatable results
        let ticket1 = BlueMorn::generate_with_seed(12345);
        let ticket2 = BlueMorn::generate_with_seed(12345);

        // Compare red balls as sets since they are sorted in the constructor
        use std::collections::HashSet;
        let red_set1: HashSet<u8> = ticket1.rball.iter().cloned().collect();
        let red_set2: HashSet<u8> = ticket2.rball.iter().cloned().collect();

        assert_eq!(red_set1, red_set2);
        assert_eq!(ticket1.bball, ticket2.bball);
    }

    #[test]
    fn test_generator_red_range() {
        let result = BlueMorn::generate_with_red_range(1, 10, Some(5));
        assert!(result.is_ok());

        let ticket = result.unwrap();
        for &red in &ticket.rball {
            assert!(red >= 1 && red <= 10);
        }
        assert_eq!(ticket.bball, 5);
    }

    #[test]
    fn test_generator_red_range_invalid() {
        let result = BlueMorn::generate_with_red_range(1, 5, Some(5));
        assert!(result.is_err());
        assert_eq!(
            result,
            Err(crate::dball::DBallError::InvalidRBallRange((1, 5)))
        );
    }

    #[test]
    fn test_multiple_tickets_check() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let tickets = vec![
            create_test_ticket([1, 2, 3, 4, 5, 6], 7), // First prize
            create_test_ticket([1, 2, 3, 4, 5, 6], 8), // Second prize
            create_test_ticket([10, 11, 12, 13, 14, 15], 8), // No win
        ];

        let results = DBall::check_multiple_tickets(&tickets, &winning_ticket);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].1, Reward::FirstPrize);
        assert_eq!(results[1].1, Reward::SecondPrize);
        assert_eq!(results[2].1, Reward::NoWin);
    }

    #[test]
    fn test_calculate_total_prize() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let tickets = vec![
            create_test_ticket([1, 2, 3, 4, 5, 6], 7),       // 4.5M
            create_test_ticket([1, 2, 3, 4, 5, 10], 7),      // 3000
            create_test_ticket([10, 11, 12, 13, 14, 15], 7), // 5
        ];

        #[allow(deprecated)]
        let total = DBall::calculate_total_prize(&tickets, &winning_ticket);
        assert_eq!(total, 4_500_000 + 3_000 + 5);
    }

    #[test]
    fn test_count_prizes() {
        let winning_ticket = create_test_ticket([1, 2, 3, 4, 5, 6], 7);
        let tickets = vec![
            create_test_ticket([1, 2, 3, 4, 5, 6], 7), // First prize
            create_test_ticket([1, 2, 3, 4, 5, 6], 8), // Second prize
            create_test_ticket([1, 2, 3, 4, 5, 6], 8), // Second prize
            create_test_ticket([10, 11, 12, 13, 14, 15], 8), // No win
        ];

        let counts = DBall::count_prizes_list(&tickets, &winning_ticket);
        assert_eq!(counts.get("#1"), Some(&1));
        assert_eq!(counts.get("#2"), Some(&2));
        assert_eq!(counts.get("#0"), Some(&1));
    }
}
