use expect_test::expect;

use crate::tests::check;

#[test]
fn objc_prose_is_c_plus_classes_selectors_and_properties() {
    check(
        "/Sources/Greeter.m",
        r##"/** Greets. */
#import <Foundation/Foundation.h>
#import "Greeter.h"
#define GREET_MAX 4

/// A greeter.
@protocol Greeting <NSObject>
- (void)greet:(NSString *)person times:(NSInteger)times;
@optional
@property (nonatomic, copy) NSString *title;
@end

@interface Greeter : NSObject <Greeting>
@property (nonatomic, strong) NSString *name;
+ (instancetype)greeterWithName:(NSString *)name;
@end

@interface Greeter ()
@property (nonatomic) int hidden;
@end

@interface Greeter (Extras)
- (void)extra;
@end

@implementation Greeter {
    int _count;
}
@synthesize name = _name;

+ (instancetype)greeterWithName:(NSString *)name {
    // Print it.
    Greeter *g = [[Greeter alloc] init];
    NSString *s = @"Hello\nworld";
    NSLog(@"recieve %@", s);
    char *c = "plain";
    return g;
}

- (void)greet:(NSString *)person times:(NSInteger)times {
    [self greetAll];
}
@end

static NSString *const kGreetKey = @"greetKey";
"##,
        expect![[r#"
            objective-c
            > /** Greets. */
                  ^^^^^^^^^^ doccomment
            > #import <Foundation/Foundation.h>
            > #import "Greeter.h"
            > #define GREET_MAX 4
                      ^^^^^^^^^ declaration
            >
            > /// A greeter.
                  ^^^^^^^^^^ doccomment
            > @protocol Greeting <NSObject>
                        ^^^^^^^^ declaration
            > - (void)greet:(NSString *)person times:(NSInteger)times;
                      ^^^^^ declaration
                                        ^^^^^^ declaration
                                               ^^^^^ declaration
                                                                ^^^^^ declaration
            > @optional
            > @property (nonatomic, copy) NSString *title;
                                                    ^^^^^ declaration
            > @end
            >
            > @interface Greeter : NSObject <Greeting>
                         ^^^^^^^ declaration
            > @property (nonatomic, strong) NSString *name;
                                                      ^^^^ declaration
            > + (instancetype)greeterWithName:(NSString *)name;
                              ^^^^^^^^^^^^^^^ declaration
                                                          ^^^^ declaration
            > @end
            >
            > @interface Greeter ()
                         ^^^^^^^ declaration
            > @property (nonatomic) int hidden;
                                        ^^^^^^ declaration
            > @end
            >
            > @interface Greeter (Extras)
                                  ^^^^^^ declaration
            > - (void)extra;
                      ^^^^^ declaration
            > @end
            >
            > @implementation Greeter {
            >     int _count;
                       ^^^^^ declaration
            > }
            > @synthesize name = _name;
            >
            > + (instancetype)greeterWithName:(NSString *)name {
                                                          ^^^^ declaration
            >     // Print it.
                  ^^^^^^^^^^^^ comment
            >     Greeter *g = [[Greeter alloc] init];
                           ^ declaration
            >     NSString *s = @"Hello\nworld";
                            ^ declaration
                                  ^^^^^ value
                                         ^^^^^ value
            >     NSLog(@"recieve %@", s);
                          ^^^^^^^^^^ string
            >     char *c = "plain";
                        ^ declaration
                             ^^^^^ value
            >     return g;
            > }
            >
            > - (void)greet:(NSString *)person times:(NSInteger)times {
                                        ^^^^^^ declaration
                                                                ^^^^^ declaration
            >     [self greetAll];
            > }
            > @end
            >
            > static NSString *const kGreetKey = @"greetKey";
                                     ^^^^^^^^^ declaration
                                                   ^^^^^^^^ value
        "#]],
    );
}

/// `.h` is both C's and Objective-C's header, and a path cannot say which, so it stays with C.
#[test]
fn a_header_path_stays_with_c_and_a_dot_m_is_objective_c() {
    let hosts = crate::Hosts::builtin();
    let id = |path: &str| hosts.for_path(path).map(|host| host.id());
    assert_eq!(id("/Sources/Greeter.h"), Some("c"));
    assert_eq!(id("/Sources/Greeter.m"), Some("objective-c"));
    assert_eq!(id("/Sources/Greeter.mm"), None);
}
