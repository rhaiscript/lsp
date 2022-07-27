module nested;

// Nested modules can be defined as well.
module inner {
    fn hey();

    module need {
        module to {
            module go_deeper {
                const YEAH;
            }
        }
    }
}
